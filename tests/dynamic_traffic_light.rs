#[macro_use]
extern crate machine;

#[derive(PartialEq,Eq,Debug,Clone)]
pub enum State {
  Green,
  Orange,
  Red,
  BlinkingOrange
}

dynamic_machine!(TrafficLight(State) {
  {
    initial: State::Green,
    error  : State::BlinkingOrange
  }

  event[next] {
    State::Green  => State::Orange,
    State::Orange => State::Red,
    State::Red    => State::Green
  }

  event[pass_car(nb: u8) -> Option<u8>: None] {
    State::Green => {
      let passed = if nb < 10 { nb } else { 10 };
      (State::Green, Some(passed))
    }
  }
});

#[test]
fn test() {
  let mut t = TrafficLight::new();
  t.pass_car(1);
  t.pass_car(2);
  t.next();
  println!("trace: {}", t.print_trace());
  assert_eq!(t.current_state(), State::Orange);

  t.next();
  assert_eq!(t.current_state(), State::Red);

  t.next();
  t.pass_car(12);
  assert_eq!(t.current_state(), State::Green);
}

