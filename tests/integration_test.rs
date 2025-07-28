use reqwest::Client;
use serde_json::json;

#[tokio::test]
async fn test_auth_and_user_features() {
    let base_url = "http://localhost:3000"; // Adjust if needed
    let client = Client::new();

    // Register user
    let register_resp = client.post(&format!("{}/auth/register", base_url))
        .json(&json!({
            "username": "testuser",
            "email": "testuser@example.com",
            "password": "password123"
        }))
        .send()
        .await
        .expect("Failed to send register request");
    assert!(register_resp.status().is_success());

    // Login user
    let login_resp = client.post(&format!("{}/auth/login", base_url))
        .json(&json!({
            "email": "testuser@example.com",
            "password": "password123"
        }))
        .send()
        .await
        .expect("Failed to send login request");
    assert!(login_resp.status().is_success());
    let login_json: serde_json::Value = login_resp.json().await.expect("Failed to parse login response");
    let token = login_json.get("token").expect("No token in login response").as_str().expect("Token is not a string");

    // Get profile
    let profile_resp = client.get(&format!("{}/user/profile", base_url))
        .bearer_auth(token)
        .send()
        .await
        .expect("Failed to send get profile request");
    assert!(profile_resp.status().is_success());
    let profile_json: serde_json::Value = profile_resp.json().await.expect("Failed to parse profile response");
    assert_eq!(profile_json.get("email").unwrap(), "testuser@example.com");

    // Update profile
    let update_resp = client.put(&format!("{}/user/profile", base_url))
        .bearer_auth(token)
        .json(&json!({
            "username": "updateduser",
            "email": "updateduser@example.com",
            "settings": null
        }))
        .send()
        .await
        .expect("Failed to send update profile request");
    assert!(update_resp.status().is_success());

    // Change password with wrong old password (should fail)
    let change_pw_fail_resp = client.put(&format!("{}/user/change-password", base_url))
        .bearer_auth(token)
        .json(&json!({
            "old_password": "wrongpassword",
            "new_password": "newpassword123"
        }))
        .send()
        .await
        .expect("Failed to send change password request");
    assert_eq!(change_pw_fail_resp.status(), 401);

    // Change password with correct old password
    let change_pw_resp = client.put(&format!("{}/user/change-password", base_url))
        .bearer_auth(token)
        .json(&json!({
            "old_password": "password123",
            "new_password": "newpassword123"
        }))
        .send()
        .await
        .expect("Failed to send change password request");
    assert!(change_pw_resp.status().is_success());

    // Logout with correct token
    let logout_resp = client.post(&format!("{}/auth/logout", base_url))
        .bearer_auth(token)
        .send()
        .await
        .expect("Failed to send logout request");
    assert!(logout_resp.status().is_success());

    // Logout with invalid token (should fail)
    let logout_fail_resp = client.post(&format!("{}/auth/logout", base_url))
        .bearer_auth("invalidtoken")
        .send()
        .await
        .expect("Failed to send logout request with invalid token");
    assert_eq!(logout_fail_resp.status(), 401);
}
