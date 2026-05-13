import { describe, it, expect, beforeEach, vi } from 'vitest';
import { useScriptStore } from '@/stores/scriptStore';
import { invoke } from '@tauri-apps/api/core';
import type { ScriptInfo } from '@/stores/scriptStore';

vi.mock('@tauri-apps/api/core');

const mockScripts: ScriptInfo[] = [
  { name: 'hello.lua', path: '/scripts/hello.lua', description: 'Hello World', modifiedAt: 1000 },
  { name: 'modbus.lua', path: '/scripts/modbus.lua', description: 'Modbus Protocol', modifiedAt: 2000 },
];

describe('scriptStore', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    useScriptStore.setState({
      scripts: [],
      currentScript: null,
      output: [],
      loading: false,
      running: false,
      error: null,
    });
  });

  describe('initial state', () => {
    it('starts with empty scripts', () => {
      const state = useScriptStore.getState();
      expect(state.scripts).toEqual([]);
    });

    it('is not running initially', () => {
      const state = useScriptStore.getState();
      expect(state.running).toBe(false);
      expect(state.loading).toBe(false);
    });
  });

  describe('loadScripts', () => {
    it('loads scripts and clears error', async () => {
      vi.mocked(invoke).mockResolvedValue(mockScripts);

      const state = useScriptStore.getState();
      await state.loadScripts();

      expect(invoke).toHaveBeenCalledWith('list_scripts');
      expect(useScriptStore.getState().scripts).toEqual(mockScripts);
      expect(useScriptStore.getState().loading).toBe(false);
      expect(useScriptStore.getState().error).toBeNull();
    });

    it('sets error on failure', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('Scripts dir not found'));

      const state = useScriptStore.getState();
      await state.loadScripts();

      expect(useScriptStore.getState().loading).toBe(false);
      expect(useScriptStore.getState().error).toBe('Scripts dir not found');
    });
  });

  describe('runScript', () => {
    it('executes script and adds result output', async () => {
      vi.mocked(invoke).mockResolvedValue('Script completed successfully');

      const state = useScriptStore.getState();
      await state.runScript('hello.lua');

      expect(invoke).toHaveBeenCalledWith('execute_script', { script: 'hello.lua' });
      expect(useScriptStore.getState().running).toBe(false);
      expect(useScriptStore.getState().currentScript).toBe('hello.lua');

      const output = useScriptStore.getState().output;
      expect(output).toHaveLength(1);
      expect(output[0].type).toBe('result');
      expect(output[0].message).toBe('Script completed successfully');
    });

    it('adds error output on script failure', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('Lua runtime error'));

      const state = useScriptStore.getState();
      await state.runScript('broken.lua');

      expect(useScriptStore.getState().running).toBe(false);
      expect(useScriptStore.getState().error).toBe('Lua runtime error');

      const output = useScriptStore.getState().output;
      expect(output).toHaveLength(1);
      expect(output[0].type).toBe('error');
      expect(output[0].message).toBe('Lua runtime error');
    });

    it('clears output before running', async () => {
      // Add some existing output
      const state = useScriptStore.getState();
      state.addOutput({ type: 'log', message: 'previous output' });
      expect(useScriptStore.getState().output).toHaveLength(1);

      vi.mocked(invoke).mockResolvedValue('done');
      await state.runScript('test.lua');

      // Should have exactly 1 output (the result), not 2
      expect(useScriptStore.getState().output).toHaveLength(1);
    });
  });

  describe('stopScript', () => {
    it('sets running to false and clears current script', () => {
      useScriptStore.setState({ running: true, currentScript: 'hello.lua' });

      const state = useScriptStore.getState();
      state.stopScript();

      const updated = useScriptStore.getState();
      expect(updated.running).toBe(false);
      expect(updated.currentScript).toBeNull();
    });
  });

  describe('addOutput', () => {
    it('adds an output line with id and timestamp', () => {
      const state = useScriptStore.getState();
      state.addOutput({ type: 'log', message: 'test message' });

      const output = useScriptStore.getState().output;
      expect(output).toHaveLength(1);
      expect(output[0].type).toBe('log');
      expect(output[0].message).toBe('test message');
      expect(output[0].id).toBeDefined();
      expect(output[0].timestamp).toBeDefined();
    });

    it('enforces 1000 line limit', () => {
      const state = useScriptStore.getState();

      for (let i = 0; i < 1005; i++) {
        state.addOutput({ type: 'log', message: `line ${i}` });
      }

      expect(useScriptStore.getState().output).toHaveLength(1000);
      // Should keep most recent lines
      expect(useScriptStore.getState().output[0].message).toBe('line 5');
      expect(useScriptStore.getState().output[999].message).toBe('line 1004');
    });
  });

  describe('clearOutput', () => {
    it('clears all output', () => {
      const state = useScriptStore.getState();
      state.addOutput({ type: 'log', message: 'test' });

      state.clearOutput();
      expect(useScriptStore.getState().output).toEqual([]);
    });
  });

  describe('saveScript', () => {
    it('saves script and refreshes list', async () => {
      vi.mocked(invoke).mockResolvedValue(undefined);

      const state = useScriptStore.getState();
      await state.saveScript('new.lua', 'print("hello")');

      expect(invoke).toHaveBeenCalledWith('save_script', {
        name: 'new.lua',
        content: 'print("hello")',
      });
      // Should refresh list after save
      expect(invoke).toHaveBeenCalledWith('list_scripts');
    });

    it('sets error on failure', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('Write failed'));

      const state = useScriptStore.getState();
      await expect(state.saveScript('fail.lua', 'content')).rejects.toThrow('Write failed');

      expect(useScriptStore.getState().error).toBe('Write failed');
    });
  });

  describe('deleteScript', () => {
    it('deletes script and refreshes list', async () => {
      vi.mocked(invoke).mockResolvedValue(undefined);

      const state = useScriptStore.getState();
      await state.deleteScript('old.lua');

      expect(invoke).toHaveBeenCalledWith('delete_script', { name: 'old.lua' });
      expect(invoke).toHaveBeenCalledWith('list_scripts');
    });

    it('sets error on failure', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('Permission denied'));

      const state = useScriptStore.getState();
      await expect(state.deleteScript('protected.lua')).rejects.toThrow('Permission denied');

      expect(useScriptStore.getState().error).toBe('Permission denied');
    });
  });
});
