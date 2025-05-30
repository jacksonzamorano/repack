import { integer, pgTable, varchar } from 'drizzle-orm/pg-core'

export const Organization = pgTable("orgs", {
	id: integer().primaryKey(),
	name: varchar(),
	email: varchar()
})



export const User = pgTable("users", {
	id: integer().primaryKey(),
	name: varchar(),
	email: varchar(),
	password: varchar(),
	org_id: integer(),
	personal_org_id: integer()
})

