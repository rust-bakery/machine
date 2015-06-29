//#![feature(log_syntax,trace_macros)]

#[macro_use] mod static_machine;
#[macro_use] mod dynamic_machine;

#[macro_export]
macro_rules! machine(
  ( $($token:tt)* ) => ( static_machine!( $($token)* ); );
);
