import type { LucideProps } from 'lucide-react'

import Icons from '#/components/icons'
import type { DbEntity } from '#/lib/taurpc'

export type WorkbenchItemType = DbEntity['kind'] | 'File' | 'Folder'

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
	Trigger: <Icons.Code className="text-teal-400" />,
	GlobalTrigger: <Icons.Code className="text-teal-400" />,
	Procedure: <Icons.Code className="text-teal-400" />,
	Index: <Icons.Code className="text-green-400" />,
	Sequence: <Icons.Code className="text-pink-300" />,
	Extension: <Icons.Code className="text-purple-400" />,
	CustomType: <Icons.Code className="text-blue-700" />,
	Schema: <Icons.Folder />,
	File: <Icons.FileCode />,
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
