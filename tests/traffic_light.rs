#[macro_use]
extern crate machine;

machine!(TrafficLight {
  attributes {
    cars: u8
  }

  impl {

  }

  states {
    Green { }  => {
      next => Orange;
    };

    Orange { } => {
      next => Red;
    };

    Red { }    =>  {
      next => Green;
    };

  }
});

impl TrafficLight<Green> {
  pub fn pass_car(&mut self) {
    self.cars = self.cars + 1;
  }
}

#[test]
fn test() {
  let mut t = TrafficLight { state: Green, cars: 0 };
  t.pass_car();
  t.pass_car();
  let t = t.next();
  assert_eq!(t, TrafficLight { state: Orange, cars: 2 } );

  let t = t.next();
  assert_eq!(t, TrafficLight { state: Red, cars: 2 } );

  let mut t = t.next();
  t.pass_car();
  assert_eq!(t, TrafficLight { state: Green, cars: 3 } );
}

