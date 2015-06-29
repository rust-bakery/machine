#[macro_use]
extern crate machine;

#[derive(PartialEq,Eq,Debug,Clone)]
pub enum State {
  Green(u8),
  Orange,
  Red,
  BlinkingOrange
}

dynamic_machine!(TrafficLight(State) {
  {
    initial: State::Green(0),
    error  : State::BlinkingOrange
  }

  attributes {
    max_passing:    u8
  }

  event[next] {
    State::Green(_) => State::Orange,
    State::Orange   => State::Red,
    State::Red      => State::Green(0)
  }

  event[pass_car(nb: u8) -> Option<u8>: None] {
    State::Green(current) => {
      let passed = if nb + current <= 10 { nb } else { 10 - current };
      if current + passed < 10 {
        (State::Green(current + passed), Some(passed))
      } else {
        (State::Orange, Some(passed))
      }
    },
    State::Orange => {
      let passed = if nb > 1 { 1 } else { nb };
      (State::Red, Some(passed))
    }
  }
});

#[test]
fn test() {
  let mut t = TrafficLight::new(10);
  t.pass_car(1);
  t.pass_car(2);
  assert_eq!(t.current_state(), State::Green(3));
  t.next();
  println!("trace: {}", t.print_trace());
  assert_eq!(t.current_state(), State::Orange);

  t.next();
  assert_eq!(t.current_state(), State::Red);

  t.next();
  assert_eq!(t.current_state(), State::Green(0));
  t.pass_car(5);
  assert_eq!(t.current_state(), State::Green(5));
  t.pass_car(7);
  assert_eq!(t.current_state(), State::Orange);
  t.pass_car(2);
  assert_eq!(t.current_state(), State::Red);
  t.reset(10);
}

