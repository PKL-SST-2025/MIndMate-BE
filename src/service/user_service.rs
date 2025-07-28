use diesel::r2d2::{self, ConnectionManager, PooledConnection};
use diesel::SqliteConnection;
use crate::db::user_query::{find_user_by_id, update_user_profile, update_user_password};
use crate::errors::app_error::AppError;
use crate::models::user::User;
use bcrypt::{verify, hash, DEFAULT_COST};

type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;
type DbConnection = PooledConnection<ConnectionManager<SqliteConnection>>;

/// Mengambil data pengguna berdasarkan ID pengguna.
///
/// # Arguments
/// * `pool` - Database connection pool.
/// * `user_id` - ID pengguna yang akan dicari.
///
/// # Returns
/// Mengembalikan `User` jika ditemukan, atau error jika tidak ditemukan atau gagal mengakses database.
pub fn get_user_by_id(pool: &DbPool, user_id: i32) -> Result<User, AppError> {
    let mut conn: DbConnection = pool.get()
        .map_err(|_| AppError::InternalServerError("Gagal mendapatkan koneksi database".to_string()))?;
    let user = find_user_by_id(&mut conn, user_id)?
        .ok_or_else(|| AppError::NotFound("Pengguna tidak ditemukan".to_string()))?;
    Ok(user)
}

/// Memperbarui profil pengguna (username dan email).
///
/// # Arguments
/// * `pool` - Database connection pool.
/// * `user_id` - ID pengguna yang akan diperbarui.
/// * `username` - Username baru.
/// * `email` - Email baru.
///
/// # Returns
/// Mengembalikan `Ok(())` jika berhasil, atau error jika gagal.
pub fn edit_profile(pool: &DbPool, user_id: i32, username: &str, email: &str) -> Result<(), AppError> {
    let mut conn: DbConnection = pool.get()
        .map_err(|_| AppError::InternalServerError("Gagal mendapatkan koneksi database".to_string()))?;
    update_user_profile(&mut conn, user_id, username, email)?;
    Ok(())
}

/// Mengubah kata sandi pengguna setelah memverifikasi kata sandi lama.
///
/// # Arguments
/// * `pool` - Database connection pool.
/// * `user_id` - ID pengguna yang akan mengubah kata sandi.
/// * `old_password` - Kata sandi lama untuk verifikasi.
/// * `new_password` - Kata sandi baru.
///
/// # Returns
/// Mengembalikan `Ok(())` jika berhasil, atau error jika kata sandi lama salah atau gagal mengakses database.
pub fn change_password(pool: &DbPool, user_id: i32, old_password: &str, new_password: &str) -> Result<(), AppError> {
    let mut conn: DbConnection = pool.get()
        .map_err(|_| AppError::InternalServerError("Gagal mendapatkan koneksi database".to_string()))?;
    
    // Verifikasi kata sandi lama
    let user = find_user_by_id(&mut conn, user_id)?
        .ok_or_else(|| AppError::NotFound("Pengguna tidak ditemukan".to_string()))?;
    
    if !verify(old_password, &user.password)
        .map_err(|_| AppError::InternalServerError("Gagal memverifikasi kata sandi".to_string()))? {
        return Err(AppError::BadRequest("Kata sandi lama salah".to_string()));
    }

    // Hash kata sandi baru sebelum menyimpan
    let hashed_password = hash(new_password, DEFAULT_COST)
        .map_err(|_| AppError::InternalServerError("Gagal menghash kata sandi baru".to_string()))?;
    
    update_user_password(&mut conn, user_id, &hashed_password)?;
    Ok(())
}