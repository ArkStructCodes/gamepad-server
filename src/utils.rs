#[macro_export]
macro_rules! axis {
    ($axis:ident, $setup:expr $(,)?) => {
        &evdev::UinputAbsSetup::new(evdev::AbsoluteAxisCode::$axis, $setup)
    };
}

#[macro_export]
macro_rules! keys {
    ($($key:ident),+ $(,)?) => {
        &evdev::AttributeSet::from_iter([$(evdev::KeyCode::$key,)*])
    };
}

#[macro_export]
macro_rules! event {
    ($event:ident, $code:expr, $value:expr $(,)?) => {
        evdev::InputEvent::new(evdev::EventType::$event.0, $code.0, $value.into())
    };
}

#[macro_export]
macro_rules! oops {
    ($kind:ident, $message:expr $(,)?) => {
        Err(std::io::Error::new(std::io::ErrorKind::$kind, $message))
    };
}
