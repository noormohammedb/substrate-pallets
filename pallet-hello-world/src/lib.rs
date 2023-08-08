#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use sp_runtime::print;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        //
    }

    // #[pallet::event]
    // pub enum Event<T: Config> {
    // }

    #[pallet::error]
    pub enum Error<T> {
        //
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        //
        pub fn say_hello(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let caller = ensure_signed(origin)?;
            print("Hello World!");
            log::info!("Request send by {:?}", caller);
            Ok(().into())
        }
    }

    impl<T: Config> Pallet<T> {
        //
    }
}
