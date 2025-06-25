import { observable } from '@legendapp/state'

import type {
	ColumnDetails,
	RowDetails,
	SchemaDetails,
	TableDetails,
} from '#/routes/-project/details/types'
import { WORKBENCH_TABS, type WorkbenchTab } from '#/routes/-project/workbench'

const uiStore$ = observable({
	devMode: false,
	workbench: {
		open: true,
		activeTab: WORKBENCH_TABS[0].label as WorkbenchTab,
	},
	bottomPanel: {
		open: false,
	},
	detailsPanel: {
		open: true,
		activeSchema: null as SchemaDetails | null,
		activeTable: null as TableDetails | null,
		activeColumn: null as ColumnDetails | null,
		activeRow: null as RowDetails | null,
	},
})

export default uiStore$
