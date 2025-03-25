import { Plus, Trash, X } from 'lucide-react';
import {
  forwardRef,
  useEffect,
  useImperativeHandle,
  useRef,
  useState,
} from 'react';
import { useTranslation } from 'react-i18next';
import { toast } from 'sonner';

import BuildInExtensions from '@/built-in-extensions.json';
import { invokeAddTool, invokeListTools } from '@/lib/commands';
import type { DialogHandler, ITool } from '@/lib/types';

import { Button } from './ui/button';
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from './ui/card';
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from './ui/dialog';
import { Input } from './ui/input';
import { Label } from './ui/label';
import { ScrollArea } from './ui/scroll-area';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from './ui/select';
import { Switch } from './ui/switch';

export const ToolsDialog = forwardRef<DialogHandler<void>, {}>((_, ref) => {
  const [showDialog, setShowDialog] = useState(false);
  const [tools, setTools] = useState<ITool[]>(BuildInExtensions as ITool[]);
  const [showAddForm, setShowAddForm] = useState(false);
  const [newTool, setNewTool] = useState<Partial<ITool>>({
    id: '',
    name: '',
    description: '',
    enabled: true,
    type: 'stdio',
    env_keys: [],
    timeout: 300,
  });
  const formRef = useRef<HTMLFormElement>(null);

  useEffect(() => {
    const fetchTools = async () => {
      const result = await invokeListTools();
      console.log('tools', result);
    };
    fetchTools();
  }, []);

  useImperativeHandle(ref, () => ({
    open: () => {
      setShowDialog(true);
    },
    close: () => {
      setShowDialog(false);
    },
  }));

  const { t } = useTranslation(['page-conversation', 'generic']);

  const handleToolToggle = (tool: ITool, enabled: boolean) => {
    // 更新工具列表中的工具状态
    setTools(tools.map((v) => (v.id === tool.id ? { ...v, enabled } : v)));

    // 显示成功提示
    toast.success(
      enabled
        ? t('page-conversation:message:tool-enabled-success')
        : t('page-conversation:message:tool-disabled-success')
    );

    // 只有启用工具时才发送到后端
    if (enabled) {
      const updatedTool = { ...tool, enabled };

      // 根据工具类型处理
      if (tool.type === 'stdio') {
        // 为命令行工具设置默认命令和参数（如果未指定）
        const toolCommand = {
          ...updatedTool,
          cmd: updatedTool.cmd || 'npx',
          args: updatedTool.args || [],
        };
        invokeAddTool(toolCommand);
      } else if (tool.type === 'sse') {
        // 检查URI是否存在
        if (updatedTool.uri) {
          invokeAddTool(updatedTool);
        } else {
          toast.error(t('page-conversation:message:tool-sse-uri-missing'));
        }
      } else if (tool.type === 'builtin') {
        // 内置工具直接添加
        invokeAddTool(updatedTool);
      }
    }
  };

  const handleDeleteTool = (name: string) => {
    // Only allow deleting non-builtin tools
    setTools(tools.filter((tool) => tool.name !== name));
    toast.success(t('page-conversation:message:tool-deleted-success'));
  };

  const handleAddTool = () => {
    // Validate form
    if (!newTool.name) {
      toast.error(t('page-conversation:message:tool-validation-error'));
      return;
    }

    // Check for duplicate ID
    if (tools.some((tool) => tool.id === newTool.id)) {
      toast.error(t('page-conversation:message:tool-id-exists-error'));
      return;
    }

    // 确保ID值存在，如果没提供则使用name转换为小写并替换空格为短横线作为ID
    const toolId = newTool.id || newTool.name?.toLowerCase().replace(/\s+/g, '-');

    // 解析超时时间，确保使用基数10
    const timeout = newTool.timeout
      ? parseInt(String(newTool.timeout), 10)
      : 300;

    // 构建工具对象
    const toolToAdd = {
      ...newTool,
      id: toolId,
      name: newTool.name,
      description: newTool.description || '',
      enabled: newTool.enabled === false ? false : true,
      type: newTool.type || 'stdio',
      env_keys: newTool.env_keys || [],
      timeout,
    } as ITool;

    // 添加到工具列表
    setTools([...tools, toolToAdd]);

    // 如果工具被启用，则调用后端API进行注册
    if (toolToAdd.enabled) {
      if (toolToAdd.type === 'stdio') {
        // 为命令行工具设置cmd和args
        if (toolToAdd.cmd) {
          invokeAddTool({
            ...toolToAdd,
            args: toolToAdd.args || [],
          });
        } else {
          toast.warning(t('page-conversation:message:tool-cmd-missing'));
        }
      } else if (toolToAdd.type === 'sse') {
        // 为SSE工具设置URI
        if (toolToAdd.uri) {
          invokeAddTool(toolToAdd);
        } else {
          toast.warning(t('page-conversation:message:tool-uri-missing'));
        }
      }
    }

    // 重置表单
    setNewTool({
      id: '',
      name: '',
      description: '',
      enabled: true,
      type: 'stdio',
      env_keys: [],
      timeout: 300,
    });

    // 隐藏表单
    setShowAddForm(false);

    toast.success(t('page-conversation:message:tool-added-success'));
  };

  return (
    <Dialog open={showDialog} onOpenChange={setShowDialog}>
      <DialogContent className="flex max-h-screen max-w-2xl flex-col">
        <DialogHeader>
          <DialogTitle className="flex items-center text-left">
            {t('page-conversation:section:tools')}
          </DialogTitle>
          <DialogDescription className="text-left">
            {t('page-conversation:message:tools-tips')}
          </DialogDescription>
        </DialogHeader>

        <div className="mb-4 flex justify-end">
          <Button
            variant="outline"
            size="sm"
            onClick={() => setShowAddForm(!showAddForm)}
          >
            {showAddForm ? (
              <X className="mr-2 size-4" />
            ) : (
              <Plus className="mr-2 size-4" />
            )}
            {showAddForm
              ? t('generic:action:cancel')
              : t('page-conversation:action:add-tool')}
          </Button>
        </div>

        {showAddForm && (
          <Card className="mb-4">
            <CardHeader>
              <CardTitle>{t('page-conversation:section:add-tool')}</CardTitle>
              <CardDescription>
                {t('page-conversation:message:add-tool-tips')}
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label htmlFor="tool-id">
                    {t('page-conversation:label:tool-id')}
                  </Label>
                  <Input
                    id="tool-id"
                    value={newTool.id || ''}
                    onChange={(e) =>
                      setNewTool({ ...newTool, id: e.target.value })
                    }
                    placeholder="unique-tool-id"
                  />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="tool-name">
                    {t('page-conversation:label:tool-name')}
                  </Label>
                  <Input
                    id="tool-name"
                    value={newTool.name || ''}
                    onChange={(e) =>
                      setNewTool({ ...newTool, name: e.target.value })
                    }
                    placeholder="My Custom Tool"
                  />
                </div>
              </div>

              <div className="space-y-2">
                <Label htmlFor="tool-description">
                  {t('page-conversation:label:tool-description')}
                </Label>
                <Input
                  id="tool-description"
                  value={newTool.description || ''}
                  onChange={(e) =>
                    setNewTool({ ...newTool, description: e.target.value })
                  }
                  placeholder="What this tool does"
                />
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label htmlFor="tool-type">
                    {t('page-conversation:label:tool-type')}
                  </Label>
                  <Select
                    value={newTool.type || 'stdio'}
                    onValueChange={(value: 'stdio' | 'sse') =>
                      setNewTool({ ...newTool, type: value })
                    }
                  >
                    <SelectTrigger id="tool-type">
                      <SelectValue placeholder="Tool type" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="stdio">
                        {t('page-conversation:option:cmd')}
                      </SelectItem>
                      <SelectItem value="sse">
                        {t('page-conversation:option:uri')}
                      </SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                <div className="space-y-2">
                  <Label htmlFor="tool-timeout">
                    {t('page-conversation:label:tool-timeout')}
                  </Label>
                  <Input
                    id="tool-timeout"
                    type="number"
                    value={newTool.timeout || 300}
                    onChange={(e) =>
                      setNewTool({
                        ...newTool,
                        timeout: parseInt(e.target.value, 10),
                      })
                    }
                    placeholder="300"
                  />
                </div>
              </div>

              {newTool.type === 'stdio' && (
                <div className="space-y-2">
                  <Label htmlFor="tool-cmd">
                    {t('page-conversation:label:tool-cmd')}
                  </Label>
                  <Input
                    id="tool-cmd"
                    value={newTool.cmd || ''}
                    onChange={(e) =>
                      setNewTool({ ...newTool, cmd: e.target.value })
                    }
                    placeholder="command to execute"
                  />
                </div>
              )}

              {newTool.type === 'sse' && (
                <div className="space-y-2">
                  <Label htmlFor="tool-uri">
                    {t('page-conversation:label:tool-uri')}
                  </Label>
                  <Input
                    id="tool-uri"
                    value={newTool.uri || ''}
                    onChange={(e) =>
                      setNewTool({ ...newTool, uri: e.target.value })
                    }
                    placeholder="https://example.com/api"
                  />
                </div>
              )}

              <div className="flex items-center space-x-2">
                <Switch
                  id="tool-enabled"
                  checked={newTool.enabled !== false}
                  onCheckedChange={(checked) =>
                    setNewTool({ ...newTool, enabled: checked })
                  }
                />
                <Label htmlFor="tool-enabled">
                  {t('page-conversation:label:tool-enabled')}
                </Label>
              </div>
            </CardContent>
            <CardFooter>
              <Button onClick={handleAddTool}>
                {t('page-conversation:action:save-tool')}
              </Button>
            </CardFooter>
          </Card>
        )}

        <ScrollArea className="max-h-[calc(100vh-400px)] grow overflow-y-auto pr-4">
          <div className="space-y-4">
            {tools.map((tool) => (
              <div
                key={tool.id}
                className="flex flex-col rounded-lg border p-4"
              >
                <div className="flex items-center justify-between">
                  <div>
                    <h3 className="text-lg font-medium">{tool.name}</h3>
                    <p className="text-sm text-muted-foreground">
                      {tool.description}
                    </p>
                    <div className="mt-1 text-xs text-muted-foreground">
                      {tool.type === 'builtin'
                        ? t('page-conversation:label:type-builtin')
                        : tool.type === 'stdio'
                        ? t('page-conversation:label:type-cmd')
                        : t('page-conversation:label:type-uri')}
                    </div>
                  </div>
                  <div className="flex items-center space-x-4">
                    <div className="flex items-center space-x-2">
                      <Switch
                        id={`toggle-${tool.id}`}
                        checked={tool.enabled}
                        onCheckedChange={(checked) =>
                          handleToolToggle(tool, checked)
                        }
                      />
                      <Label htmlFor={`toggle-${tool.id}`}>
                        {tool.enabled
                          ? t('generic:state:enabled')
                          : t('generic:state:disabled')}
                      </Label>
                    </div>
                    {tool.type !== 'builtin' && (
                      <Button
                        variant="ghost"
                        size="icon"
                        onClick={() => handleDeleteTool(tool.name)}
                      >
                        <Trash className="size-4" />
                      </Button>
                    )}
                  </div>
                </div>
              </div>
            ))}
          </div>
        </ScrollArea>

        <DialogFooter className="mt-4">
          <DialogClose asChild>
            <Button variant="secondary">{t('generic:action:close')}</Button>
          </DialogClose>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
});

ToolsDialog.displayName = 'ToolsDialog';
