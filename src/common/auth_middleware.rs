// // TODO 暂时直接放开
// // TODO 判断token存在，游客可以参观的路由,登录后将token放入header中
// let token = token.to_str().unwrap();
// tracing::info!("{}", token);
// let decode = util::token_decode(token).await;
// tracing::info!("{:#?}", decode);
// if decode.pk_id > 0
// || req.path().to_string() == "/login"
// || req.path().to_string() == "/captcha"
// {
// Ok(svc.call(req).await?)
// } else {
// Err(error::ErrorUnauthorized("您未登录，请登录后使用此功能"))
