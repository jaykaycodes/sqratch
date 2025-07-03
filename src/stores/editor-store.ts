import { batch, observable } from '@legendapp/state'
import { LRUCache } from 'lru-cache'

export type EditorTabType = 'table' | 'file'
export type EditorTabId = `${EditorTabType}_${string}`

export interface EditorTab {
	id: EditorTabId
	resourceId: string
	resourceType: EditorTabType
	name: string
	isTemp?: boolean
	isDirty?: boolean
}

const activeTabHistory = new LRUCache<EditorTabId, number>({
	max: 100,
	ttl: Infinity,
})

const EditorStore$ = observable({
	tabs: new Map<EditorTabId, EditorTab>(),
	tabOrder: [] as EditorTabId[],
	activeTabId: null as EditorTabId | null,
	activeTab: () => {
		const tabId = EditorStore$.activeTabId.get()
		return tabId ? EditorStore$.tabs.get(tabId) : null
	},
	getTabHistory: () => Array.from(activeTabHistory.rkeys()),
	closeTab: (id: EditorTabId) => EditorStore$.tabs.delete(id),
	openTab(resourceType: EditorTabType, resourceId: string, atIndex?: number) {
		const id: EditorTabId = `${resourceType}_${resourceId}`
		const tabOrder = EditorStore$.tabOrder.get()

		batch(() => {
			if (!EditorStore$.tabs.has(id)) {
				EditorStore$.tabs.set(id, {
					id,
					resourceType,
					resourceId,
				})
				EditorStore$.tabOrder.set([
					...tabOrder.slice(0, atIndex),
					id,
					...tabOrder.slice(atIndex ?? tabOrder.length),
				])
			}

			EditorStore$.activeTabId.set(id)
		})

		return id
	},
})

export default EditorStore$

// track active tab history
EditorStore$.activeTabId.onChange(({ getPrevious, value }) => {
	if (value && getPrevious() !== value) activeTabHistory.set(value, Date.now())
})

EditorStore$.tabs.onChange(({ changes, getPrevious, value }) => {
	if (getPrevious().size === 0 && value.size > 0) {
		// going from 0->1 tabs activate by default
		const nextTab = value.keys().next().value
		if (nextTab) EditorStore$.activeTabId.set(nextTab)
	}

	for (const change of changes) {
		// on tab removal
		if (change.prevAtPath?.id && !change.valueAtPath) {
			// remove tab from tab ordering
			EditorStore$.tabOrder.set(
				EditorStore$.tabOrder.peek().filter((id) => id !== change.prevAtPath.id),
			)

			// delete tab from history
			activeTabHistory.delete(change.prevAtPath.id)

			// set next recent tab as active
			const nextRecentTab = activeTabHistory.rkeys().next().value
			if (nextRecentTab) {
				EditorStore$.activeTabId.set(nextRecentTab)
			} else {
				// if no more recent tabs, set the first tab as active
				EditorStore$.activeTabId.set(EditorStore$.tabOrder.peek()[0])
			}
		}
	}
})
