import { timestamp, varchar, integer, bigint, pgTable, doublePrecision } from 'drizzle-orm/pg-core'

export const Organization = pgTable("orgs", {
	id: integer().notNull().primaryKey().generatedAlwaysAsIdentity(),
	name: varchar().notNull(),
	email: varchar().notNull()
})



export const User = pgTable("users", {
	id: integer().notNull().primaryKey().generatedAlwaysAsIdentity(),
	name: varchar().notNull(),
	email: varchar().notNull(),
	password: varchar().notNull(),
	login_count: bigint({ mode: 'number' }).notNull(),
	last_login: timestamp({ withTimezone: true }).notNull(),
	total_cost: doublePrecision().notNull(),
	org_id: integer().notNull().references(() => Organization.id),
	personal_org_id: integer().notNull().references(() => Organization.id)
})

