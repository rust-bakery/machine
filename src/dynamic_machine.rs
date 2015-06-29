#[macro_export]
macro_rules! dynamic_machine (
  ( $machine:ident($state:ty) {
    {
      initial: $initial:expr,
      error:   $error:expr
    }

    attributes {
      $($name:ident : $t:ty),*
    }

    $(
    event[$($def:tt)*] {
      $($tokens:tt)*
    }
    )*
  }) => (
    #[derive(PartialEq,Eq,Clone,Debug)]
    struct $machine {
      state: $state,
      trace: Vec<(&'static str, $state)>,
      $($name : $t),*
    }

    #[allow(dead_code)]
    impl $machine {
      fn new($($name : $t),*) -> $machine {
        $machine { state: $initial, trace: vec![ ("", $initial) ], $($name : $name),* }
      }

      $(transitions!(
          $error,
          $($def)*,
          $($tokens)*
        );
      )*

      fn reset(&mut self, $($name : $t),*) {
        self.state = $initial;
        self.trace = vec![ ("", $initial) ];
        $(self.$name = $name;)*
      }

      fn is_invalid(&self) -> bool {
        self.state == $error
      }

      fn current_state(&self) -> $state {
        self.state.clone()
      }

      fn push_state(&mut self, event: &'static str, st: $state) {
        self.trace.push((event, st))
      }

      fn print_trace(&self) -> String {
        let mut res = String::new();
        res.push_str(stringify!($initial));
        for & (event, ref state) in (&self.trace[1..]).iter() {
          res.push_str(& format!(" =( {} )=> {:?}", event, state));
        }
        res
      }
    }
  )
);

#[macro_export]
macro_rules! transitions (
  ($err:expr,  $ev:ident, $($state:pat => $res:expr),*) => (
    fn $ev(&mut self) -> Option<()> {
      match self.state {
        $($state => {
          self.push_state(stringify!($ev), $res.clone());
          self.state = $res;
          Some(())
        },)*
        _        => {
          self.state = $err;
          self.push_state(stringify!($ev), $err);
          None
        }
      }
    }
  );
  ($err:expr,  $ev:ident ($($args:ident : $t:ty),*) -> $out:ty : $default:expr, $($state:pat => $b:block),*) => (
    fn $ev(&mut self, $($args:$t),*) -> $out {
      match self.state {
        $($state => {
          let (new_state, result) = $b;
          self.push_state(stringify!($ev), new_state.clone());
          self.state = new_state;
          result
        }),*
        _        => {
          self.state = $err;
          self.push_state(stringify!($ev), $err);
          $default
        }
      }
    }
  );
);

#[cfg(test)]
mod tests {

  #[derive(PartialEq,Eq,Debug,Clone)]
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

  //trace_macros!(true);
  dynamic_machine!(Machine(State) {
    {
      initial: State::A,
      error  : State::Error
    }

    attributes { }

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
  //trace_macros!(false);

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
    let res2 = m.tr3(12);
    println!("4: state({:?}): {:?}", res2, m);
    let mut res3 = m.tr4(1);
    println!("5: state({:?}): {:?}", res3, m);
    res3 = m.tr4(1);
    println!("6: state({:?}): {:?}", res3, m);
    res = m.tr2();
    println!("7: state({:?}): {:?}", res, m);
    if m.is_invalid() {
      println!("had an invalid transition");
    }
    m.tr();
    m.tr2();
    println!("trace: {}", m.print_trace());
    m.reset();
    println!("8: state: {:?}", m);
    //assert!(false);
  }
}
