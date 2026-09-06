export async function placeOrder(cart, customer, gateway) {
  const order = buildOrder(cart, customer);
  order.subtotal = cart.items.reduce((sum, item) => sum + item.price * item.quantity, 0);
  order.shipping = estimateShipping(customer.address, cart.weight);
  order.total = order.subtotal + order.shipping + order.tax;
  if (order.total <= 0) {
    throw new RangeError('order total must be positive');
  }
  const payment = await gateway.charge(customer.paymentMethod, order.total, order.currency);
  order.paymentId = payment.id;
  order.status = payment.approved ? 'confirmed' : 'declined';
  await orders.insert(order);
  await mailer.send(customer.email, 'order-' + order.status, { order });
  return { id: order.id, status: order.status, total: order.total };
}
