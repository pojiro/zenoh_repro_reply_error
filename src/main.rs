fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use zenoh::Wait;

    #[test]
    fn repro_many_times() {
        for i in 0..10000 {
            println!("Repetition #{}", i);
            repro();
        }
    }

    fn repro() {
        let mut config = zenoh::config::Config::default();
        config.insert_json5("scouting/delay", "0").unwrap();
        let session = zenoh::session::open(config).wait().unwrap();

        let (tx_query, rx_query) = std::sync::mpsc::channel();
        let (tx_reply, rx_reply) = std::sync::mpsc::channel();

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

        {
            let query = rx_query.recv().unwrap();

            query.reply("key/expr/1", "1").wait().unwrap();
            query.reply_err("2").wait().unwrap();
            query.reply("key/expr/3", "3").wait().unwrap();
        }

        let replies =
            rx_reply.iter().fold(
                Vec::<String>::new(),
                |mut vec: Vec<String>, reply| match reply.result() {
                    Ok(sample) => {
                        vec.push(format!("{}", sample.payload().try_to_string().unwrap()));
                        vec
                    }
                    Err(reply_error) => {
                        vec.push(format!(
                            "{}",
                            reply_error.payload().try_to_string().unwrap()
                        ));
                        vec
                    }
                },
            );

        let actual: HashSet<_> = replies.into_iter().collect();

        let expected: HashSet<_> = vec!["1".to_string(), "2".to_string(), "3".to_string()]
            .into_iter()
            .collect();

        assert_eq!(actual, expected);
    }
}
