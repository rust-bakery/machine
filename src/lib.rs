#![feature(log_syntax,trace_macros)]

#[macro_export]
macro_rules! machine (
  ( $machine:ident($state:ty) {
    {
      initial: $initial:path,
      error:   $error:path
    }

    $(
    event[$($def:tt)*] {
      $($tokens:tt)*
    }
    )*
  }) => (
    #[derive(PartialEq,Eq,Debug)]
    struct $machine {
      state: $state
    }

    impl $machine {
      fn new() -> $machine {
        $machine { state: $initial }
      }

      $(transitions!(
          $error,
          $($def)*,
          $($tokens)*
        );
      )*
    }
  )
);

macro_rules! transitions (
  ($err:path,  $ev:ident, $($state:pat => $res:expr),*) => (
    fn $ev(&mut self) -> Option<()> {
      match self.state {
        $($state => {self.state = $res; Some(())},)*
        _        => {self.state = $err; None}
      }
    }
  );
  ($err:path,  $ev:ident ($($args:ident : $t:ty),*) -> $out:ty : $default:expr, $($state:pat => $b:block),*) => (
    fn $ev(&mut self, $($args:$t),*) -> $out {
      match self.state {
        $($state => {
          let (new_state, result) = $b;
          self.state = new_state;
          result
        }),*
        _        => {self.state = $err; $default}
      }
    }
  );
);

#[cfg(test)]
mod tests {
  #![feature(trace_macros)]
  use super::*;

  #[derive(PartialEq,Eq,Debug)]
  pub enum State {
    A, B(u8), C(u8), Error
  }

  #[derive(PartialEq,Eq,Debug)]
  pub enum Parsed {
    X(u8),
    Y,
    Z
  }

  pub fn parse(arg:u8) -> (State, Parsed) {
    if arg > 10 {
      (State::B(arg), Parsed::X(arg))
    } else {
      (State::Error, Parsed::Y)
    }
  }

  trace_macros!(true);
  machine!(Machine(State) {
    {
      initial: State::A,
      error  : State::Error
    }

    event[tr]{
      State::A    => State::B(0),
      State::B(i) => State::C(i+1)
    }

    event[tr2] {
      State::C(_) => State::A,
      State::A    => State::C(42)
    }

    event [tr3(arg1:u8) -> Option<u8> : None] {
      State::A    => { (State::B(arg1), Some(42)) },
      State::B(i) => { (State::C(i+1), Some(i+1)) }
    }

    event [tr4(arg1:u8) -> Parsed: Parsed::Z] {
      State::A => {
        parse(arg1)
      },
      State::B(i) => {
        parse(i+1)
      }
    }
   });
  trace_macros!(false);

  #[test]
  fn a() {
    let mut m = Machine::new();
    println!("0: state: {:?}", m);
    let mut res = m.tr();
    println!("1: state({:?}): {:?}", res, m);
    res = m.tr();
    println!("2: state({:?}): {:?}", res, m);
    res = m.tr2();
    println!("3: state({:?}): {:?}", res, m);
    let mut res2 = m.tr3(12);
    println!("4: state({:?}): {:?}", res2, m);
    let mut res3 = m.tr4(1);
    println!("5: state({:?}): {:?}", res3, m);
    res3 = m.tr4(1);
    println!("6: state({:?}): {:?}", res3, m);
    assert!(false);
  }
}
