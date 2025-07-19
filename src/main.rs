use zenoh::Wait;

/*
 * RUST_LOG="trace" cargo run
 */

fn main() {
    let mut config = zenoh::config::Config::default();
    config.insert_json5("scouting/delay", "0").unwrap();
    let session = zenoh::session::open(config).wait().unwrap();

    let (tx_query, rx_query) = std::sync::mpsc::channel();
    let (tx_reply, _rx_reply) = std::sync::mpsc::channel();

    env_logger::init();

    let _queryable = session
        .declare_queryable("key/expr")
        .callback(move |query| {
            tx_query.send(query).unwrap();
        })
        .wait()
        .unwrap();

    session
        .get("key/expr/**")
        .callback(move |reply| {
            tx_reply.send(reply).unwrap();
        })
        .wait()
        .unwrap();

    log::warn!("====================================================");
    {
        let query = rx_query.recv().unwrap();
        log::warn!("====================================================");
        query.reply("key/expr/1", [1u8]).wait().unwrap();
        query.reply_err([2u8]).wait().unwrap();
        query.reply("key/expr/3", [3u8]).wait().unwrap();
    }

    /*
       let _replies = rx_reply
           .iter()
           .fold(Vec::<u8>::new(), |mut vec: Vec<u8>, reply| {
               match reply.result() {
                   Ok(sample) => {
                       vec.extend_from_slice(&sample.payload().to_bytes());
                       vec
                   }
                   Err(reply_error) => {
                       vec.extend_from_slice(&reply_error.payload().to_bytes());
                       vec
                   }
               }
           });
    */
}
