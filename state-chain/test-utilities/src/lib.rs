use frame_system::Config;

pub fn last_event<T: Config>() -> <T as Config>::RuntimeEvent {
	maybe_last_event::<T>().expect("Event expected")
}

pub fn maybe_last_event<T: Config>() -> Option<<T as Config>::RuntimeEvent> {
	frame_system::Pallet::<T>::events().pop().map(|e| e.event)
}

/// Can be used to check that fixed-sized types have the correct implementation of MaxEncodedLen
pub fn ensure_max_encoded_len_is_exact<T: Default + codec::Encode + codec::MaxEncodedLen>() {
	assert_eq!(T::default().encode().len(), T::max_encoded_len());
}

/// Checks the deposited events in the order they occur
#[macro_export]
macro_rules! assert_event_sequence {
	($runtime:ty, $($evt:expr),*) => {
		let mut events = frame_system::Pallet::<$runtime>::events()
		.into_iter()
		// We want to be able to input the events into this macro in the order they occurred.
		.rev()
		.map(|e| e.event)
			.collect::<Vec<_>>();

		$(
			let actual = events.pop().expect("Expected an event.");
			assert_eq!(actual, $evt);
		)*
	};
}

#[macro_export]
macro_rules! assert_has_event_pattern {
	($( $pattern:pat_param )|+ $( if $guard: expr )? ) => {
		assert!(
			System::events().iter().any(|record| matches!(record.event, $( $pattern )|+ $( if $guard )?)),
			"No event that matches {}. Available events: {:#?}",
			stringify!($( $pattern )|+ $( if $guard )?),
			System::events().into_iter().map(|record| record.event).collect::<Vec<_>>()
		)
	};
}

#[macro_export]
macro_rules! extract_from_event {
	( $pattern:pat => $bind:expr ) => {
		System::events()
			.into_iter()
			.filter_map(|record| if let $pattern = record.event { Some($bind) } else { None })
			.next()
			.expect(&format!("No event that matches {}", stringify!($pattern))[..])
	};
}
