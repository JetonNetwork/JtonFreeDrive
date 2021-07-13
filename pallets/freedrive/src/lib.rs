#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>

use sp_std::prelude::*;
use sp_runtime::{DispatchResult, traits::StaticLookup};

use frame_support::{
	weights::GetDispatchInfo,
	traits::UnfilteredDispatchable,
};

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;


#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::*;

	// important to use outside structs and consts
	use super::*;

	type Calls = u32;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// A free drive call.
		type Call: Parameter + UnfilteredDispatchable<Origin=Self::Origin> + GetDispatchInfo;
		// Max calls allowed per account and session.
		type MaxCalls: Get<u32>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://substrate.dev/docs/en/knowledgebase/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://substrate.dev/docs/en/knowledgebase/runtime/storage#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;

	#[pallet::storage]
	#[pallet::getter(fn tracker)]
	/// Store players active board, currently only one board per player allowed.
	pub type Tracker<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, (T::BlockNumber, Calls), ValueQuery>;

	// Default value for session length
	#[pallet::type_value]
	pub fn SessionLengthDefault<T: Config>() -> T::BlockNumber { 1000u32.into() }
	#[pallet::storage]
	#[pallet::getter(fn session_length)]
	pub type SessionLength<T: Config> = StorageValue<_, T::BlockNumber, ValueQuery, SessionLengthDefault<T>>;

	// Pallets use events to inform users when important changes are made.
	// https://substrate.dev/docs/en/knowledgebase/runtime/events
	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),
		/// Result of the free drive extrinsic
		ExtrinsicResult(T::AccountId, DispatchResult),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		//User already as used free drive calls.
		NoFreeCalls,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T:Config> Pallet<T> {
		/// Free drive ...
		///
		/// The dispatch origin for this call must be _Signed_.
		///
		/// # <weight>
		/// - O(1).
		/// - Limited storage reads.
		/// - One DB write (event).
		/// - Weight of derivative `call` execution + 10,000.
		/// # </weight>
		#[pallet::weight({
			let dispatch_info = call.get_dispatch_info();
			(dispatch_info.weight.saturating_add(10_000), dispatch_info.class)
		})]
		pub fn freedrive(
			origin: OriginFor<T>,
			call: Box<<T as Config>::Call>,
		) -> DispatchResultWithPostInfo {
			// This is a public call, so we ensure that the origin is some signed account.
			let sender = ensure_signed(origin.clone())?;
			//let res = call.dispatch_bypass_filter(frame_system::RawOrigin::Root.into());

			let max_calls = T::MaxCalls::get();

			//let user_calls = Self::tracker(&sender);
			let (last_user_session, mut user_calls) = <Tracker<T>>::get(&sender);

			let current_block_number = <frame_system::Pallet<T>>::block_number();
			let session_length = <SessionLength<T>>::get();
			let current_session = current_block_number / session_length.into();

			if last_user_session < current_session {
				user_calls = 0;
			}

			if user_calls < max_calls {
				<Tracker<T>>::insert(&sender, (current_session, user_calls.saturating_add(1)));
				//Tracker::<T>::insert(&sender, user_calls.saturating_add(1));
	
				let res = call.dispatch_bypass_filter(origin);
	
				// trigger event when successfully made a free drive call
				Self::deposit_event(Event::<T>::ExtrinsicResult(sender, res.map(|_| ()).map_err(|e| e.error)));

				return Ok(Pays::No.into())

			} else {
				let check_logic_weight = T::DbWeight::get().reads(3);

				return Ok(Some(check_logic_weight).into())
			}

		}
	}
}
