import {
	ArrowsClockwise,
	FolderOpen,
	MagnifyingGlass,
	Sparkle,
	Trash
} from '@phosphor-icons/react';
import type {Location} from '@sd/ts-client';
import {useNavigate} from 'react-router-dom';
import {useLibraryMutation} from '../../../contexts/SpacedriveContext';
import {
	useContextMenu,
	type ContextMenuItem,
	type ContextMenuResult
} from '../../../hooks/useContextMenu';

interface UseLocationContextMenuOptions {
	location: Location;
}

/**
 * Specialized context menu for Location items in the sidebar.
 * Includes: Open, Rescan (Quick/Full), and Remove.
 */
export function useLocationContextMenu({
	location
}: UseLocationContextMenuOptions): ContextMenuResult {
	const navigate = useNavigate();
	const rescanLocation = useLibraryMutation('locations.rescan');
	const removeLocation = useLibraryMutation('locations.remove');

	const items: ContextMenuItem[] = [
		{
			icon: FolderOpen,
			label: 'Open',
			onClick: () => {
				navigate(`/explorer?path=Location:${location.id}`);
			}
		},
		{type: 'separator'},
		{
			icon: ArrowsClockwise,
			label: 'Quick Rescan',
			onClick: () => {
				rescanLocation.mutate({
					location_id: location.id,
					full_rescan: false
				});
			}
		},
		{
			icon: Sparkle,
			label: 'Full Rescan',
			onClick: () => {
				rescanLocation.mutate({
					location_id: location.id,
					full_rescan: true
				});
			}
		},
		{type: 'separator'},
		{
			icon: Trash,
			label: 'Remove Location',
			onClick: async () => {
				// Note: In a real app, this might show a confirmation dialog first.
				// For now, we match the Inspector's direct mutation capability.
				try {
					await removeLocation.mutateAsync({
						location_id: String(location.id)
					});
				} catch (err) {
					console.error('Failed to remove location:', err);
				}
			},
			variant: 'danger'
		}
	];

	return useContextMenu({items});
}
