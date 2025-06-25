import { observable } from '@legendapp/state'

export type EditorTabType = 'entity' | 'query'
export interface EditorTab {
	type: EditorTabType
	id: string
	name: string
	isTemp?: boolean
	isDirty?: boolean
}

const EditorStore$ = observable({
	tabs: [] as EditorTab[],
	focusedTab: null as number | null,
	openTab(type: EditorTabType, id: string, name: string) {
		// TODO use legend state tricks to keep active synced
		const tab = EditorStore$.tabs.find((tab) => tab.get().id === id)
		if (tab) {
			EditorStore$.focusedTab.set(EditorStore$.tabs.indexOf(tab))
			return
		}

		EditorStore$.tabs.push({
			type,
			id,
			name,
		})
	},
})

export default EditorStore$
