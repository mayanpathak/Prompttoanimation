









// tests/src/test_users.rs

use fake::{
    faker::{
        address::en::CountryName,
        internet::en::{FreeEmail, Password},
        name::en::{FirstName, LastName},
    },
    Fake,
};

use rand::{rngs::StdRng, Rng, SeedableRng};
use serde_json::{json, Value};
use uuid::Uuid;

pub fn generate_test_users(count: usize) -> Vec<Value> {
    let mut rng = StdRng::from_entropy();

    (0..count)
        .map(|_| {
            // Generate realistic names
            let first_name: String =
                FirstName().fake_with_rng(&mut rng);

            let last_name: String =
                LastName().fake_with_rng(&mut rng);

            // Realistic username styles
            let username_styles = vec![
                format!(
                    "{}{}",
                    first_name.to_lowercase(),
                    rng.gen_range(100..9999)
                ),

                format!(
                    "{}_{}",
                    first_name.to_lowercase(),
                    last_name.to_lowercase()
                ),

                format!(
                    "{}.{}{}",
                    first_name.to_lowercase(),
                    last_name.to_lowercase(),
                    rng.gen_range(10..999)
                ),

                format!(
                    "{}{}{}",
                    first_name
                        .chars()
                        .next()
                        .unwrap()
                        .to_lowercase(),
                    last_name.to_lowercase(),
                    rng.gen_range(1000..9999)
                ),
            ];

            let username = username_styles
                [rng.gen_range(0..username_styles.len())]
                .clone();

            // Generate realistic email domain
            let random_email: String =
                FreeEmail().fake_with_rng(&mut rng);

            let domain = random_email
                .split('@')
                .last()
                .unwrap();

            let email = format!(
                "{}@{}",
                username,
                domain
            );

            // Strong random password
            let password: String =
                Password(12..18)
                    .fake_with_rng(&mut rng);

            // Country
            let country: String =
                CountryName()
                    .fake_with_rng(&mut rng);

            json!({
                "id": Uuid::new_v4().to_string(),

                "first_name": first_name,
                "last_name": last_name,

                "username": username,
                "email": email,
                "password": password,

                "country": country,

                "age": rng.gen_range(18..75),

                "is_active": rng.gen_bool(0.92),
                "is_verified": rng.gen_bool(0.78),

                "login_count": rng.gen_range(0..250),

                "created_at": chrono::Utc::now()
                    .to_rfc3339(),
            })
        })
        .collect()
}

pub fn get_user_credentials(
    count: usize,
) -> Vec<(String, String)> {
    generate_test_users(count)
        .iter()
        .map(|user| {
            let email = user["email"]
                .as_str()
                .unwrap()
                .to_string();

            let password = user["password"]
                .as_str()
                .unwrap()
                .to_string();

            (email, password)
        })
        .collect()
}