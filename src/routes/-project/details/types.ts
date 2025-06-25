import type { Entity } from '#/lib/taurpc'

export interface SchemaDetails extends Entity {}

export interface TableDetails extends Entity {}

export interface ColumnDetails {
	id: string
	name: string
	dataType: string
	sizeBytes: number
}

export interface RowDetails {
	[name: string]: ColumnDetails
}
