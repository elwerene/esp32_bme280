use anyhow::Result;
use embedded_svc::http::server::{registry::Registry, Handler, HandlerResult, Request, Response};
use embedded_svc::io::Write;
use esp_idf_svc::http::server::EspHttpServer;
use std::io::ErrorKind;

pub struct Httpd(EspHttpServer);

impl Httpd {
    pub fn setup() -> Result<Self> {
        let mut server = EspHttpServer::new(&Default::default())?;
        server.set_handler("/sessions", embedded_svc::http::Method::Get, Sessions {})?;

        Ok(Self(server))
    }
}

struct Sessions {}

impl<R: Request, S: Response> Handler<R, S> for Sessions {
    fn handle(&self, req: R, resp: S) -> HandlerResult {
        let sessions = crate::sessions::sessions(req.query_string() == "delete")?;
        serde_json::to_writer(EmbeddedIoWriter(resp.into_writer()?), &sessions)?;

        Ok(())
    }
}

pub struct EmbeddedIoWriter<W: Write>(pub W);

impl<W: Write> std::io::Write for EmbeddedIoWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0
            .write(buf)
            .map_err(|_err| std::io::Error::new(ErrorKind::Other, "Error writing"))
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.0
            .flush()
            .map_err(|_err| std::io::Error::new(ErrorKind::Other, "Error flushing"))
    }
}
