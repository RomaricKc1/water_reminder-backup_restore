#[derive(Debug)]
pub enum RouteListenErr {
    BindError,
}

#[derive(Debug)]
pub enum TuiError {
    BadIp,
    BadPort,
    Other,
}
