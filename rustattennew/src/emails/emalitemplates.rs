pub const VERIFICATION_EMAIL_TEMPLATE: &str = r#"
<!DOCTYPE html>
<html>
<body>
  <h2>Verify Your Email</h2>
  <p>Your verification code is:</p>
  <h1>{verificationCode}</h1>
</body>
</html>
"#;

pub const PASSWORD_RESET_REQUEST_TEMPLATE: &str = r#"
<!DOCTYPE html>
<html>
<body>
  <h2>Password Reset</h2>
  <p>Click below to reset:</p>
  <a href="{resetURL}">Reset Password</a>
</body>
</html>
"#;

pub const PASSWORD_RESET_SUCCESS_TEMPLATE: &str = r#"
<!DOCTYPE html>
<html>
<body>
  <h2>Password Reset Successful</h2>
  <p>Your password has been updated.</p>
</body>
</html>
"#;