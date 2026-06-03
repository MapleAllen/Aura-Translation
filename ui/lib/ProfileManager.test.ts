import { fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import ProfileManager from './ProfileManager.svelte';
import type { ApiKeyStorage, Provider } from './appConfig';
import type { TranslationProfile, TranslationProfilesStore } from './translationProfiles';

const baseProfile: TranslationProfile = {
  id: 'profile-1',
  name: 'Default',
  provider: 'deepseek' satisfies Provider,
  model: 'deepseek-chat',
  source_lang: 'auto',
  target_lang: 'Chinese',
  api_key_storage: 'system' satisfies ApiKeyStorage,
  api_base_url: 'https://api.deepseek.com',
  available_models: ['deepseek-chat', 'deepseek-reasoner'],
  api_key: '',
};

const secondaryProfile = {
  ...baseProfile,
  id: 'profile-2',
  name: 'Work',
  target_lang: 'English',
};

const multiStore: TranslationProfilesStore = {
  active_profile_id: 'profile-1',
  profiles: [baseProfile, secondaryProfile],
};

const singleStore: TranslationProfilesStore = {
  active_profile_id: 'profile-1',
  profiles: [baseProfile],
};

describe('ProfileManager delete inline confirm', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('does not call ondelete on the first click of Delete', async () => {
    const ondelete = vi.fn();
    render(ProfileManager, {
      store: multiStore,
      draftName: 'Default',
      ondelete,
    });

    const deleteButtons = screen.getAllByTestId('delete-profile-button');
    await fireEvent.click(deleteButtons[0]);
    expect(ondelete).not.toHaveBeenCalled();
  });

  it('renders profiles as compact operator rows', () => {
    render(ProfileManager, {
      store: multiStore,
      draftName: 'Default',
    });

    expect(screen.getAllByTestId('profile-row')).toHaveLength(2);
    expect(screen.getByTestId('profile-list')).toHaveTextContent('Default');
    expect(screen.getByTestId('profile-list')).toHaveTextContent('Work');
  });

  it('shows the confirm group inline and hides the original button', async () => {
    render(ProfileManager, {
      store: multiStore,
      draftName: 'Default',
    });

    const deleteButtons = screen.getAllByTestId('delete-profile-button');
    await fireEvent.click(deleteButtons[0]);

    expect(screen.getByTestId('delete-confirm-group')).toBeInTheDocument();
    expect(screen.getByTestId('delete-confirm-commit')).toHaveTextContent('确认删除');
    expect(screen.getByTestId('delete-confirm-cancel')).toHaveTextContent('取消');
  });

  it('calls ondelete only when the user confirms', async () => {
    const ondelete = vi.fn();
    render(ProfileManager, {
      store: multiStore,
      draftName: 'Default',
      ondelete,
    });

    const deleteButtons = screen.getAllByTestId('delete-profile-button');
    await fireEvent.click(deleteButtons[0]);
    await fireEvent.click(screen.getByTestId('delete-confirm-commit'));

    expect(ondelete).toHaveBeenCalledTimes(1);
    expect(ondelete).toHaveBeenCalledWith('profile-1');
  });

  it('cancels without calling ondelete when Cancel is pressed', async () => {
    const ondelete = vi.fn();
    render(ProfileManager, {
      store: multiStore,
      draftName: 'Default',
      ondelete,
    });

    const deleteButtons = screen.getAllByTestId('delete-profile-button');
    await fireEvent.click(deleteButtons[0]);
    await fireEvent.click(screen.getByTestId('delete-confirm-cancel'));

    expect(ondelete).not.toHaveBeenCalled();
    expect(screen.getAllByTestId('delete-profile-button')).toHaveLength(2);
  });

  it('auto-resets to the initial state after the 5s timeout', async () => {
    const ondelete = vi.fn();
    render(ProfileManager, {
      store: multiStore,
      draftName: 'Default',
      ondelete,
    });

    const deleteButtons = screen.getAllByTestId('delete-profile-button');
    await fireEvent.click(deleteButtons[0]);
    expect(screen.getByTestId('delete-confirm-group')).toBeInTheDocument();

    vi.advanceTimersByTime(5000);

    await waitFor(() => {
      expect(screen.queryByTestId('delete-confirm-group')).not.toBeInTheDocument();
    });
    expect(ondelete).not.toHaveBeenCalled();
  });

  it('keeps the Delete button disabled when only one profile remains', () => {
    const ondelete = vi.fn();
    render(ProfileManager, {
      store: singleStore,
      draftName: 'Default',
      ondelete,
    });

    const deleteButton = screen.getByTestId('delete-profile-button');
    expect(deleteButton).toBeDisabled();
  });

  it('isolates confirm state per row so confirming one row does not affect another', async () => {
    const ondelete = vi.fn();
    render(ProfileManager, {
      store: multiStore,
      draftName: 'Default',
      ondelete,
    });

    const deleteButtons = screen.getAllByTestId('delete-profile-button');
    await fireEvent.click(deleteButtons[0]);

    expect(screen.getByTestId('delete-confirm-group')).toBeInTheDocument();
    expect(deleteButtons[1]).toBeInTheDocument();
    expect(deleteButtons[1]).not.toBeDisabled();
  });
});
