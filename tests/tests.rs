//! Copyright © Marko Vujnovic, GNU Affero General Public License v3

use rustscript::*;

#[test]
fn run_main_with_args_t() -> core::result::Result<(), std::io::Error> { tokio::runtime::Runtime::new().unwrap().block_on(async {
    let script_path = "./example.mpa";
    main_(&script_path).await?;
    Ok(())
})}

#[test]
fn get_unix_username_t() -> core::result::Result<(), std::io::Error> { tokio::runtime::Runtime::new().unwrap().block_on(async {
    let should_be = format!("{}", std::str::from_utf8(&async_process::Command::new("whoami").output().await?.stdout).unwrap());
    // println!("\"{}\"", should_be);
    assert!(uid_to_username(1000).unwrap() == should_be[0..should_be.len() - 1] );
    Ok(())
})}

