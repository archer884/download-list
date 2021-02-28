use std::{
    fs,
    fmt::Write,
    io::{self, Read},
    time::Duration,
};

use sha2::{Digest, Sha256};
use ureq::AgentBuilder;

fn main() -> io::Result<()> {
    let mut buf = String::new();
    let queue = read_stdin(&mut buf)?;

    let agent = AgentBuilder::new().timeout(Duration::from_secs(20)).build();

    for item in queue {
        let response = match agent.get(item).call() {
            Ok(response) => response,
            Err(_) => {
                eprintln!("{}", item);
                continue;
            }
        };

        let mut buf = Vec::new();
        let filename = response
            .header("content-disposition")
            .and_then(extract_filename);

        if response.into_reader().read_to_end(&mut buf).is_err() {
            eprintln!("{}", item);
            continue;
        }

        let filename = filename.unwrap_or_else(|| filename_from_sha2(&buf));
        fs::write(filename, &buf)?;
    }

    Ok(())
}

fn extract_filename(disposition: &str) -> Option<String> {
    // inline;filename="a45752e6fd3d3297614f4242f6f8779cab42f630-1S_1280.jpg"

    let filename_segment = disposition
        .split(';')
        .filter(|&segment| segment.starts_with("filename"))
        .next()?;

    let filename = filename_segment[filename_segment.find('=')? + 1..]
        .trim_matches('"')
        .to_owned();

    Some(filename)
}

fn filename_from_sha2(data: &[u8]) -> String {
    let mut digest = Sha256::new();
    let mut formatted = String::new();
    
    digest.update(data);

    for u in digest.finalize() {
        let _ = write!(formatted, "{:02x}", u);
    }
    formatted
}

fn read_stdin(buf: &mut String) -> io::Result<impl Iterator<Item = &str>> {
    io::stdin().read_to_string(buf)?;
    Ok(buf.lines())
}

#[cfg(test)]
mod tests {
    #[test]
    fn extract_filename() {
        let disposition = r#"inline;filename="a45752e6fd3d3297614f4242f6f8779cab42f630-1S_1280.jpg";"#;
        let actual = super::extract_filename(disposition).unwrap();
        let expected = "a45752e6fd3d3297614f4242f6f8779cab42f630-1S_1280.jpg";
        assert_eq!(actual, expected);
    }

    #[test]
    fn filename_from_sha2() {
        let actual = super::filename_from_sha2(b"1234");
        let expected = "03ac674216f3e15c761ee1a5e255f067953623c8b388b4459e13f978d7c846f4";
        assert_eq!(actual, expected);
    }
}
