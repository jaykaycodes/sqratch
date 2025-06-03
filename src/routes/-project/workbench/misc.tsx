import type { LucideProps } from 'lucide-react'

import Icons from '#/components/icons'
import type { Entity, EntityType } from '#/lib/taurpc'

export type WorkbenchItemType = EntityType | 'File' | 'Folder'

export interface WorkbenchItem {
	id: string
	type: WorkbenchItemType
	name: string
	path: string[]
}

export const itemIcons = {
	Table: <Icons.Table className="text-yellow-300" />,
	ForeignTable: <Icons.Table className="text-purple-400" />,
	View: <Icons.LayoutGrid className="text-blue-300" />,
	MaterializedView: <Icons.LayoutGrid className="text-blue-500" />,
	Function: <Icons.Code className="text-teal-400" />,
	Procedure: <Icons.Code className="text-teal-400" />,
	Sequence: <Icons.Code className="text-pink-300" />,
	Type: <Icons.Code className="text-blue-700" />,
	Schema: <Icons.Folder />,
	File: <Icons.FileCode className="text-green-400" />,
	Folder: <Icons.Folder />,
} satisfies Record<WorkbenchItemType, React.ReactNode>

export interface WorkbenchTabConfig {
	label: string
	Icon: React.FC<LucideProps>
	emptyText: string
}

export const WORKBENCH_TABS = [
	{
		label: 'Database',
		Icon: Icons.Database,
		emptyText: 'No database entities found',
	},
	{
		label: 'Queries',
		Icon: Icons.FileCode2,
		emptyText: 'No queries found',
	},
] as const satisfies WorkbenchTabConfig[]
export type WorkbenchTab = (typeof WORKBENCH_TABS)[number]['label']

export function mapEntities(entities: Entity[]): WorkbenchItem[] {
	if (!entities || !entities.length) return []

	const [publicItems, otherItems] = entities
		.sort((a, b) => {
			const aFullPath = [a.schema_name, a.name].filter(Boolean).join('.')
			const bFullPath = [b.schema_name, b.name].filter(Boolean).join('.')
			return aFullPath.localeCompare(bFullPath)
		})
		.reduce(
			([publicItems, otherItems], entity) => {
				if (entity.schema_name === 'public') publicItems.push(entity)
				else otherItems.push(entity)
				return [publicItems, otherItems]
			},
			[[] as Entity[], [] as Entity[]],
		)

	const mapEntity = (entity: Entity): WorkbenchItem => ({
		id: entity.id,
		name: entity.name,
		type: entity.entity_type,
		path: entity.schema_name && entity.schema_name !== 'public' ? [entity.schema_name] : [],
	})

	return [...publicItems.map(mapEntity), ...otherItems.map(mapEntity)]
}
