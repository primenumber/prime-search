extern crate iron;
extern crate router;
extern crate staticfile;
#[macro_use]
extern crate lazy_static;
extern crate urlencoded;

use iron::mime::Mime;
use iron::prelude::*;
use iron::status;
use mount::Mount;
use router::Router;
use staticfile::Static;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::str::FromStr;
use urlencoded::UrlEncodedBody;

const SQRTN: usize = 4096;
const N: usize = SQRTN * SQRTN;
lazy_static! {
    static ref PRIME_LIST: Vec<usize> = prime_list();
    static ref COUNT: Vec<usize> = {
        let mut res = Vec::new();
        let f = File::open("table").expect("table not found");
        for line in BufReader::new(f).lines() {
            res.push(line.unwrap().parse::<usize>().unwrap());
        }
        res
    };
    static ref PARTIAL_SUM: Vec<usize> = {
        let m = COUNT.len();
        let mut res = vec![0; m + 1];
        for i in 0usize..m {
            res[i + 1] = res[i] + COUNT[i];
        }
        res
    };
}

fn prime_list() -> Vec<usize> {
    let mut is_prime = vec![true; N + 1];
    is_prime[0] = false;
    is_prime[1] = false;
    for i in 2usize..(N + 1) {
        if i * i > N {
            break;
        }
        if is_prime[i] {
            let mut j = i * i;
            while j <= N {
                is_prime[j] = false;
                j += i;
            }
        }
    }
    is_prime
        .iter()
        .enumerate()
        .filter_map(|(i, is_p)| {
            if *is_p {
                return Some(i);
            } else {
                return None;
            }
        })
        .collect()
}

fn prime_list_range(begin: usize, size: usize) -> Vec<usize> {
    let mut is_prime = vec![true; size];
    for i in PRIME_LIST.iter() {
        if i * i >= begin + size {
            break;
        }
        let mut j = (i - (begin % i)) % i;
        while j < size {
            if begin + j > *i {
                is_prime[j] = false;
            }
            j += i;
        }
    }
    is_prime
        .iter()
        .enumerate()
        .filter_map(|(i, is_p)| {
            if begin + i <= 1 {
                return None;
            } else if *is_p {
                return Some(begin + i);
            } else {
                return None;
            }
        })
        .collect()
}

fn main() {
    println!("number of primes under n is: {}", PRIME_LIST.len());
    println!("100000th prime is: {}", nth_prime(1000000));

    let mut router = Router::new();

    router.post("/nth_prime", post_nth_prime, "nth_prime");
    router.post("/prime_count", post_prime_count, "prime_count");

    let mut mount = Mount::new();
    mount.mount("/", Static::new(Path::new("static/")));
    mount.mount("/api/", router);

    //fn handler(req: &mut Request) -> IronResult<Response> {
    //  println!("OK: {}", req.url);
    //  Ok(Response::new())
    //}
    //router.get("/:query", handler, "handler");

    let port = 3210;
    println!("Serving on http://localhost:{}...", port);
    Iron::new(mount)
        .http(format!("localhost:{}", port))
        .unwrap();
}

fn post_nth_prime(request: &mut Request) -> IronResult<Response> {
    println!("post_nth_prime");
    let mut response = Response::new();

    let form_data = match request.get_ref::<UrlEncodedBody>() {
        Err(e) => {
            response.set_mut(status::BadRequest);
            response.set_mut("application/json".parse::<Mime>().unwrap());
            response.set_mut(format!(
                r#"{{"status":"error", "message":"Error parsing form data: {:?}"}}\n"#,
                e
            ));
            return Ok(response);
        }
        Ok(map) => map,
    };

    let unparsed_numbers = match form_data.get("n") {
        None => {
            response.set_mut(status::BadRequest);
            response.set_mut("application/json".parse::<Mime>().unwrap());
            response.set_mut(format!(
                r#"{{"status":"error", "message":"Form data has no 'n' parameter"}}\n"#
            ));
            return Ok(response);
        }
        Some(nums) => nums,
    };

    if unparsed_numbers.len() != 1 {
        response.set_mut(status::BadRequest);
        response.set_mut("application/json".parse::<Mime>().unwrap());
        response.set_mut(format!(
            r#"{{"status":"error", "message":"number of 'n' must be 1"}}"#
        ));
        return Ok(response);
    }

    let number = match usize::from_str(&unparsed_numbers[0]) {
        Err(_) => {
            response.set_mut(status::BadRequest);
            response.set_mut("application/json".parse::<Mime>().unwrap());
            response.set_mut(format!(
                r#"{{"status":"error", "message":"Value for 'n' parameter not a number: {}"}}"#,
                unparsed_numbers[0]
            ));
            return Ok(response);
        }
        Ok(x) => x,
    };

    if number > *PARTIAL_SUM.iter().last().unwrap() {
        response.set_mut(status::BadRequest);
        response.set_mut("application/json".parse::<Mime>().unwrap());
        response.set_mut(format!(
            r#"{{"status":"error", "message":"Too Large number: {}"}}"#,
            number
        ));
        return Ok(response);
    }

    if number == 0 {
        response.set_mut(status::BadRequest);
        response.set_mut("application/json".parse::<Mime>().unwrap());
        response.set_mut(r#"{"status":"error", "message":"Zero is not accepted"}"#);
        return Ok(response);
    }

    let p = nth_prime(number);

    response.set_mut(status::Ok);
    response.set_mut("application/json".parse::<Mime>().unwrap());
    response.set_mut(format!(r#"{{"status":"ok", "result":{}}}"#, p));

    Ok(response)
}

fn nth_prime(x: usize) -> usize {
    let m = COUNT.len();
    let mut lo = 0;
    let mut hi = m;
    while (hi - lo) > 1 {
        let mid = (lo + hi) / 2;
        if x > PARTIAL_SUM[mid] {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    if lo > 0 {
        let v = prime_list_range(N * lo, N);
        let rem = x - PARTIAL_SUM[lo];
        v[rem - 1]
    } else {
        PRIME_LIST[x - 1]
    }
}

fn post_prime_count(request: &mut Request) -> IronResult<Response> {
    println!("post_prime_count");
    let mut response = Response::new();

    let form_data = match request.get_ref::<UrlEncodedBody>() {
        Err(e) => {
            response.set_mut(status::BadRequest);
            response.set_mut("application/json".parse::<Mime>().unwrap());
            response.set_mut(format!(
                r#"{{"status":"error", "message":"Error parsing form data: {:?}"}}\n"#,
                e
            ));
            return Ok(response);
        }
        Ok(map) => map,
    };

    let unparsed_numbers = match form_data.get("n") {
        None => {
            response.set_mut(status::BadRequest);
            response.set_mut("application/json".parse::<Mime>().unwrap());
            response.set_mut(format!(
                r#"{{"status":"error", "message":"Form data has no 'n' parameter"}}\n"#
            ));
            return Ok(response);
        }
        Some(nums) => nums,
    };

    if unparsed_numbers.len() != 1 {
        response.set_mut(status::BadRequest);
        response.set_mut("application/json".parse::<Mime>().unwrap());
        response.set_mut(format!(
            r#"{{"status":"error", "message":"number of 'n' must be 1"}}"#
        ));
        return Ok(response);
    }

    let number = match usize::from_str(&unparsed_numbers[0]) {
        Err(_) => {
            response.set_mut(status::BadRequest);
            response.set_mut("application/json".parse::<Mime>().unwrap());
            response.set_mut(format!(
                r#"{{"status":"error", "message":"Value for 'n' parameter not a number: {}"}}"#,
                unparsed_numbers[0]
            ));
            return Ok(response);
        }
        Ok(x) => x,
    };

    if number > COUNT.len() * N {
        response.set_mut(status::BadRequest);
        response.set_mut("application/json".parse::<Mime>().unwrap());
        response.set_mut(format!(
            r#"{{"status":"error", "message":"Too Large number: {}"}}"#,
            number
        ));
        return Ok(response);
    }

    let p = prime_count(number);

    response.set_mut(status::Ok);
    response.set_mut("application/json".parse::<Mime>().unwrap());
    response.set_mut(format!(r#"{{"status":"ok", "result":{}}}"#, p));
    println!(r#"{{"status":"ok", "result":{}}}"#, p);

    Ok(response)
}

fn prime_count(x: usize) -> usize {
    let row = x / N;
    let col = x % N;
    PARTIAL_SUM[row] + prime_list_range(N * row, col + 1).len()
}
