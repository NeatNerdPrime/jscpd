export function buildCreditNote(refund, account, vatRate) {
  const entries = [];
  for (const item of refund.items) {
    if (!item.refundable) continue;
    const net = item.price * item.quantity;
    entries.push({ sku: item.sku, quantity: item.quantity, net });
  }
  const subtotal = entries.reduce((sum, entry) => sum + entry.net, 0);
  const vat = Math.round(subtotal * vatRate * 100) / 100;
  logger.info('credit note', { account: account.id, subtotal });
  return {
    number: nextCreditNoteNumber(),
    account: account.id,
    entries,
    subtotal,
    vat,
    total: subtotal + vat,
  };
}
