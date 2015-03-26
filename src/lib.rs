#![feature(trace_macros)]

#[macro_export]
macro_rules! machine (
  ( $machine:ident {
      $($rest:tt)*
    }
  ) => (
    #[derive(PartialEq,Eq,Debug)]
    pub struct $machine<T> {
      state: T
    }

    state_impl!($machine, $($rest)* );
  )
);

#[macro_export]
macro_rules! state_impl (
  ($machine:ident, $state:ident {  } => { $($transitions:tt)* }; $($rest:tt)* ) => (
    #[derive(PartialEq,Eq,Debug)]
    pub struct $state;

    impl $machine<$state> {
      transitions_impl!($machine, $state, $($transitions)*);
    }

    state_impl!($machine, $($rest)*);
  );

  ($machine:ident, $state:ident { $($name:ident : $t:ty),* } => { $($transitions:tt)* }; $($rest:tt)*) => (
    #[derive(PartialEq,Eq,Debug)]
    pub struct $state { $($name : $t,)* }

    impl $machine<$state> {
      transitions_impl!($machine, $state, $($transitions)*);
    }

    state_impl!($machine, $($rest)*);
  );

  ($machine:ident, $state:ident {  }; $($rest:tt)* ) => (
    #[derive(PartialEq,Eq,Debug)]
    pub struct $state;

    state_impl!($machine, $($rest)*);
  );

  ($machine:ident, $state:ident { $($name:ident : $t:ty),* }; $($rest:tt)*) => (
    #[derive(PartialEq,Eq,Debug)]
    pub struct $state { $($name : $t,)* }

    state_impl!($machine, $($rest)*);
  );

  ($machine:ident,) =>  ( );
);

#[macro_export]
macro_rules! transitions_impl (
  ($machine:ident, $state:ident,) => (
  );

  ($machine:ident, $state:ident, $name:ident ( $( $arg:ident : $t:ty ),* ) => $next:ident $b:block; $($rest:tt)* ) => (
    trans_impl!($machine, $state, $name ( $( $arg : $t ),* ) => $next $b;);
    transitions_impl!($machine, $state, $($rest)*);
  );
  ($machine:ident, $state:ident, $name:ident => $next:ident; $($rest:tt)* ) => (

    trans_impl!($machine, $state, $name => $next;);
    transitions_impl!($machine, $state, $($rest)*);
  );
  ($machine:ident, $state:ident, $name:ident => $next:ident $b:block; $($rest:tt)* ) => (

    trans_impl!($machine, $state, $name  => $next $b;);
    transitions_impl!($machine, $state, $($rest)*);
  );

);

#[macro_export]
macro_rules! trans_impl (
  ($machine:ident, $state:ident, $name:ident => $next:ident;) => (
    pub fn $name(&self) -> $machine<$next> {
      return $machine {
        state: $next
      }
    }
  );
  ($machine:ident, $state:ident, $name:ident ( $( $arg:ident : $t:ty ),* ) => $next:ident $b:block; ) => (
    pub fn $name(&self,  $( $arg : $t ),* ) -> $machine<$next> {
      let res = $b;
      return $machine {
        state: res
      }
    }
  );
  ($machine:ident, $state:ident, $name:ident => $next:ident $b:block;) => (
    pub fn $name(&self) -> $machine<$next> {
      let res = $b;
      return $machine {
        state: res
      }
    }
  );
);
#[cfg(test)]
mod tests {
  #![feature(trace_macros)]
  use super::*;

  trace_macros!(true);
  machine!(Machine {
    SA { } => {
      tr1(a:u8) => SB
      {
        println!("current: {}", a);
        SB { b: a, c: false }
      };

    };

    SB {
      b: u8,
      c: bool
    } => {
      tr2 => SC;
      tr3 => SA {
        SA
      };
    };

    SC {

    } => {
      tr1 => SA;
    };
  });
  trace_macros!(false);

  #[test]
  fn transitions() {
    let m = Machine { state: SB { b: 1, c: true } };
    let m = m.tr3();
    assert_eq!(m, Machine { state: SA });
    let m = m.tr1(42);
    assert_eq!(m, Machine { state: SB { b:42, c:false } });
    let m = m.tr2().tr1();
    assert_eq!(m, Machine { state: SA });
  }
}
