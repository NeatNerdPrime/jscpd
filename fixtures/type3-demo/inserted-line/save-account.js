export async function saveUser(user, db, clock) {
  const row = toRow(user);
  row.updatedAt = clock.now();
  row.version = (row.version || 0) + 1;
  row.normalizedEmail = user.email.trim().toLowerCase();
  if (!row.id) throw new Error('cannot save a user without an id');
  row.displayName = [user.firstName, user.lastName].filter(Boolean).join(' ');
  await db.put('users', row.id, row);
  await audit('save', row.id, row.version, clock.now());
  await notify(user.email, 'profile-updated', { version: row.version });
  return { id: row.id, version: row.version, updatedAt: row.updatedAt };
}
