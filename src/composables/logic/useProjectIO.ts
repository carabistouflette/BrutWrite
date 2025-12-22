import { projectApi } from '../../api/project';
import { useAppStatus } from '../useAppStatus';
import { useCharacters } from '../useCharacters';
import { reconstructHierarchy } from '../../utils/tree';
import type { ProjectSettings, Plotline } from '../../types';
import { 
    projectData, 
    projectId, 
    activeId, 
    projectSettings, 
    projectPlotlines 
} from '../state/projectState';
import { useProjectNodeOperations } from './useProjectNodeOperations';

export function useProjectIO() {
    const { notifyError } = useAppStatus();
    const { addChapter } = useProjectNodeOperations();

    const updateRecentProjects = (path: string) => {
        const recentStr = localStorage.getItem('recent_projects') || '[]';
        let recent: string[] = JSON.parse(recentStr);
        recent = [path, ...recent.filter(p => p !== path)].slice(0, 5);
        localStorage.setItem('recent_projects', JSON.stringify(recent));
    };

    const loadProject = async (path: string) => {
        try {
            const metadata = await projectApi.load(path);
            projectId.value = metadata.id;

            // Sync characters to their dedicated store
            const { setCharacters } = useCharacters();
            setCharacters(metadata.characters);

            // Set settings
            projectSettings.value = metadata.settings;
            projectPlotlines.value = metadata.plotlines;

            localStorage.setItem('last_opened_project_path', path);
            updateRecentProjects(path);

            projectData.value = reconstructHierarchy(metadata.manifest.chapters);

            if (projectData.value.length > 0) {
                activeId.value = projectData.value[0].id;
            }
        } catch (e) {
            notifyError('Failed to load project', e);
            localStorage.removeItem('last_opened_project_path');
        }
    };

    const createProject = async (path: string, name: string, author: string) => {
        try {
            const metadata = await projectApi.create(path, name, author);
            projectId.value = metadata.id;
            projectData.value = [];

            // Reset character store
            const { setCharacters } = useCharacters();
            setCharacters([]);

            projectSettings.value = metadata.settings;
            projectPlotlines.value = metadata.plotlines;

            localStorage.setItem('last_opened_project_path', path);
            updateRecentProjects(path);

            await addChapter();
        } catch (e) {
            notifyError('Failed to create project', e);
        }
    }

    const closeProject = () => {
        activeId.value = undefined;
        projectData.value = [];
        projectId.value = undefined;
        projectSettings.value = null;
        projectPlotlines.value = [];

        const { setCharacters } = useCharacters();
        setCharacters([]);

        localStorage.removeItem('last_opened_project_path');
    };

    const updateSettings = async (settings: ProjectSettings) => {
        if (!projectId.value) return;
        try {
            const metadata = await projectApi.updateSettings(projectId.value, settings);
            projectSettings.value = metadata.settings;
        } catch (e) {
            notifyError('Failed to update project settings', e);
        }
    };

    const updatePlotlines = async (plotlines: Plotline[]) => {
        if (!projectId.value) return;
        try {
            const metadata = await projectApi.updatePlotlines(projectId.value, plotlines);
            projectPlotlines.value = metadata.plotlines;
        } catch (e) {
            notifyError('Failed to update plotlines', e);
        }
    };

    return {
        loadProject,
        createProject,
        closeProject,
        updateSettings,
        updatePlotlines
    };
}
