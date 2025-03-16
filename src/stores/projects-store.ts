import { z } from 'zod'

import { atomWithTypedStorage } from '#/lib/jotai'

// Define the schema for a single database connection
export const DbConnectionSchema = z.object({
  id: z.string(),
  name: z.string(),
  connectionString: z.string(),
  host: z.string(),
  port: z.number(),
  database: z.string(),
  user: z.string(),
  password: z.string().optional(),
  createdAt: z.number(),
})
export type DbConnection = z.infer<typeof DbConnectionSchema>

// Create the atom for storing connections
export const connectionsAtom = atomWithTypedStorage<
  z.ZodArray<typeof DbConnectionSchema>,
  DbConnection[],
  DbConnection[]
>('db-connections', DbConnectionSchema.array(), [])
