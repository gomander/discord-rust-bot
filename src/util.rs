use serenity::all::User;

pub fn get_user_name(user: &User) -> String {
    user.global_name
        .as_deref()
        .unwrap_or(&user.name)
        .to_string()
}
