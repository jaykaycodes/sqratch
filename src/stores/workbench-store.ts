import type { TreeState } from '@headless-tree/core'
import { observable } from '@legendapp/state'
import type { Query } from '@tanstack/react-query'

import type { DbEntity } from '#/lib/taurpc'
import { WORKBENCH_TABS, type WorkbenchTab } from '#/routes/-project/workbench'

const WorkbenchStore$ = observable({
	open: true,
	activeTab: WORKBENCH_TABS[0].label as WorkbenchTab,
	treeStates: {
		database: {
			selectedItems: [] as string[],
			expandedItems: [] as string[],
			focusedItem: null as string | null,
		} satisfies Partial<TreeState<DbEntity>>,
		queries: {} as Partial<TreeState<Query>>,
	},
})

export default WorkbenchStore$
