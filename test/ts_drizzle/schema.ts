import { pgTable, bigint, doublePrecision, varchar, integer, timestamp, boolean } from 'drizzle-orm/pg-core'

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



export const List = pgTable("lists", {
	id: integer().notNull().primaryKey().generatedAlwaysAsIdentity(),
	title: varchar().notNull(),
	description: varchar().notNull(),
	creator_user_id: integer().notNull().references(() => User.id),
	org_id: integer().notNull().references(() => Organization.id)
})



export const Todo = pgTable("todos", {
	id: integer().notNull().primaryKey().generatedAlwaysAsIdentity(),
	done: boolean().notNull(),
	title: varchar().notNull(),
	description: varchar().notNull(),
	creator_user_id: integer().notNull().references(() => User.id),
	assigned_user_id: integer().references(() => User.id),
	list_id: integer().references(() => List.id),
	org_id: integer().notNull().references(() => Organization.id)
})

