use common::Amount;

mod common;
mod limit_orders;
mod range_orders;

pub struct PoolState {
	pub limit_orders: limit_orders::PoolState,
	pub range_orders: range_orders::PoolState,
}
impl PoolState {
	pub fn swap<SD: common::SwapDirection + limit_orders::SwapDirection + range_orders::SwapDirection>(&mut self, mut amount: Amount) -> (Amount, Amount) {
		let mut total_output_amount = Amount::zero();

		while amount != Amount::zero() {
			let (output_amount, remaining_amount) = match (self.limit_orders.current_sqrt_price::<SD>(), self.range_orders.current_sqrt_price::<SD>()) {
				(Some(limit_order_sqrt_price), Some(range_order_sqrt_price)) => {
					if SD::sqrt_price_op_more_than(limit_order_sqrt_price, range_order_sqrt_price) {
						self.range_orders.swap::<SD>(amount, Some(limit_order_sqrt_price))
					} else {
						self.limit_orders.swap::<SD>(amount, Some(range_order_sqrt_price))
					}
				},
				(Some(_), None) => self.limit_orders.swap::<SD>(amount, None),
				(None, Some(_)) => self.range_orders.swap::<SD>(amount, None),
				(None, None) => break,
			};

			amount = remaining_amount;
			total_output_amount = total_output_amount.saturating_add(output_amount);
		}

		(total_output_amount, amount)
	}
}