#[macro_use]
extern crate machine;

machine!(
    enum HttpRequest {
        Initial,
        HasRequestLine {
            request: RequestLine,
        },
        HasHost {
            request: RequestLine,
            host: String,
        },
        HasLength {
            request: RequestLine,
            length: LengthInfo,
        },
        HasHostAndLength {
            request: RequestLine,
            host: Host,
            length: LengthInfo,
        },
        Request {
            request: RequestLine,
            host: Host,
        },
        RequestWithBody {
            request: RequestLine,
            host: Host,
            remaining: usize,
        },
        RequestWithChunks {
            request: RequestLine,
            host: Host,
            chunk: ChunkState,
        },
    }
);

#[derive(Clone, Debug, PartialEq)]
pub struct RequestLine;

#[derive(Clone, Debug, PartialEq)]
pub struct HostHeader(String);

#[derive(Clone, Debug, PartialEq)]
pub struct LengthHeader(LengthInfo);

#[derive(Clone, Debug, PartialEq)]
pub struct HeaderEnd;

pub type Host = String;

#[derive(Clone, Debug, PartialEq)]
pub enum LengthInfo {
    Length(usize),
    Chunked,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ChunkState;

transitions!(HttpRequest,
  [
    (Initial, RequestLine) => HasRequestLine,
    (HasRequestLine, HostHeader) => HasHost,
    (HasRequestLine, LengthHeader) => HasLength,
    (HasHost, LengthHeader) => HasHostAndLength,
    (HasLength, HostHeader) => HasHostAndLength,
    (HasHost, HeaderEnd) => Request,
    (HasHostAndLength, HeaderEnd) => [RequestWithBody, RequestWithChunks]
  ]
);

methods!(HttpRequest,
  [
    HasHost, HasHostAndLength, Request,
      RequestWithBody, RequestWithChunks => get host: str
  ]
);

impl Initial {
  pub fn on_request_line(self, request: RequestLine) -> HasRequestLine {
    HasRequestLine { request }
  }
}

impl HasRequestLine {
  pub fn on_host_header(self, h: HostHeader) -> HasHost {
    let HostHeader(host) = h;

    HasHost {
      request: self.request,
      host,
    }
  }

  pub fn on_length_header(self, h: LengthHeader) -> HasLength {
    let LengthHeader(length) = h;

    HasLength {
      request: self.request,
      length,
    }
  }
}

impl HasHost {
  pub fn on_length_header(self, h: LengthHeader) -> HasHostAndLength {
    let LengthHeader(length) = h;

    HasHostAndLength {
      request: self.request,
      host: self.host,
      length,
    }
  }

  pub fn on_header_end(self, _: HeaderEnd) -> Request {
    Request {
      request: self.request,
      host: self.host,
    }
  }
}

impl HasLength {
  pub fn on_host_header(self, h: HostHeader) -> HasHostAndLength {
    let HostHeader(host) = h;

    HasHostAndLength {
      request: self.request,
      length: self.length,
      host,
    }
  }
}

impl HasHostAndLength {
  pub fn on_header_end(self, _: HeaderEnd) -> HttpRequest {
    match self.length {
      LengthInfo::Length(remaining) => HttpRequest::RequestWithBody(RequestWithBody {
        request: self.request,
        host: self.host,
        remaining
      }),
      LengthInfo::Chunked => {
        HttpRequest::RequestWithChunks(RequestWithChunks {
          request: self.request,
          host: self.host,
          chunk: ChunkState,
        })
      }
    }
  }
}
