export function buildInvoice(order, customer, taxRate) {
  const lines = [];
  for (const item of order.items) {
    const net = item.price * item.quantity;
    lines.push({ sku: item.sku, quantity: item.quantity, net });
  }
  const subtotal = lines.reduce((sum, line) => sum + line.net, 0);
  const tax = Math.round(subtotal * taxRate * 100) / 100;
  return {
    number: nextInvoiceNumber(),
    customer: customer.id,
    lines,
    subtotal,
    tax,
    total: subtotal + tax,
  };
}
