import { observable } from '@legendapp/state'
import { isHotkeyPressed } from 'react-hotkeys-hook'

import Log from '#/lib/logger'

export type DetailsPanelTab = 'entity' | 'column' | 'row'
export interface DetailsPanelItem {
	type: DetailsPanelTab
	id: string
}

const DetailsPanelStore$ = observable({
	open: true,
	activeTab: 'entity' as DetailsPanelTab,
	selectedItems: [] as DetailsPanelItem[],
	selectItem(...items: DetailsPanelItem[]) {
		const type = items[0].type

		// Enforce all items have the same type
		if (items.some((item) => item.type !== type)) {
			Log.warn('DetailsPanelStore$.selectItem: Mismatched item types, using first item only')
			DetailsPanelStore$.selectedItems.set(items)
		}

		if (isHotkeyPressed('shift')) {
			DetailsPanelStore$.selectedItems.push(...items)
		} else {
			DetailsPanelStore$.selectedItems.set([items[0]])
		}

		DetailsPanelStore$.activeTab.set(type)
	},
})

export default DetailsPanelStore$
