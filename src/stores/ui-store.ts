import { observable } from '@legendapp/state'

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
		activeObjectId: undefined as string | undefined,
	},
})

export default uiStore$
