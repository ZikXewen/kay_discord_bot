macro_rules! reg_err {
    ($ctx: ident, $x: expr) => {
        match $x {
            Ok(val) => val,
            Err(err) => {
                crate::utils::replies::say_error($ctx, err.to_string()).await;
                return Ok(());
            }
        }
    };
    ($ctx: ident, $x: expr, $err: expr) => {
        match $x {
            Ok(val) => val,
            Err(_) => {
                crate::utils::replies::say_error($ctx, $err).await;
                return Ok(());
            }
        }
    };
}

macro_rules! msg_err {
    ($x: expr) => {
        match $x {
            Ok(val) => val,
            Err(err) => {
                eprintln!("{}", err);
                return Ok(());
            }
        }
    };
}

pub(crate) use msg_err;
pub(crate) use reg_err;
