import type { SchemaEntityType } from '#/lib/taurpc'

export type WorkbenchItemType = SchemaEntityType | 'UserQuery'

interface WorkbenchItemBase {
	id: string
	name: string
}

export interface WorkbenchItem extends WorkbenchItemBase {
	type: WorkbenchItemType
}

export interface WorkbenchSchemaGroup extends WorkbenchItemBase {
	type: 'Schema'
	items: WorkbenchItem[]
}

export interface WorkbenchFolderGroup extends WorkbenchItemBase {
	type: 'Folder'
	items: (WorkbenchItem | WorkbenchFolderGroup)[]
}

export type WorkbenchGroup = WorkbenchFolderGroup | WorkbenchSchemaGroup
export type WorkbenchQueryItems = (WorkbenchItem | WorkbenchFolderGroup)[]
export type WorkbenchItemOrGroup = WorkbenchItem | WorkbenchGroup
