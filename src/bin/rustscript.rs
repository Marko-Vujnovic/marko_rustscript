//! Copyright © Marko Vujnovic, GNU Affero General Public License v3

use rustscript::*;

fn main() -> core::result::Result<(), std::io::Error> { tokio::runtime::Runtime::new().unwrap().block_on(async {
    let (options, positional_args) = parse_args(std::env::args());
    let script_path = &positional_args[1];
    main_(&script_path).await?;
    Ok(())
})}

