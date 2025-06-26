import { observable } from '@legendapp/state'

import logger from '#/lib/logger'

import WorkbenchStore$ from './workbench-store'

export interface DetailsPanelItem {
	type: 'entity' | 'row'
	id: string
}

const DetailsPanelStore$ = observable({
	open: true,
	selectedItems: [] as DetailsPanelItem[],
	selectItem(...items: DetailsPanelItem[]) {
		const newType = items[0].type

		// Enforce all items have the same type
		if (items.some((item) => item.type !== newType)) {
			logger.warn('DetailsPanelStore$.selectItem: Mismatched item types, using first item only')
			DetailsPanelStore$.selectedItems.set(items)
		}

		DetailsPanelStore$.selectedItems.set(items)
	},
})

export default DetailsPanelStore$

WorkbenchStore$.treeStates.database.selectedItems.onChange(({ value: items }) =>
	DetailsPanelStore$.selectItem(...items.map((item) => ({ type: 'entity' as const, id: item }))),
)
