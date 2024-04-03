import { PaperPlaneIcon } from '@radix-ui/react-icons';
import { useQueryClient } from '@tanstack/react-query';
import { useRef } from 'react';
import { toast } from 'sonner';

import { MESSAGE_USER } from '@/lib/constants';
import { LIST_MESSAGES_KEY, useCreateMessageMutation } from '@/lib/hooks';
import type { Message } from '@/lib/types';

import { Button } from './ui/button';
import { Textarea } from './ui/textarea';

const HEIGHT_LIMIT = 20 * 20;

type Props = {
  conversationId: number;
};

export function ChatPromptInput({ conversationId }: Props) {
  const promptRef = useRef<HTMLTextAreaElement>(null);
  const queryClient = useQueryClient();
  const createMsgMutation = useCreateMessageMutation();

  // Callbacks
  const onChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    const ta = e.target as HTMLTextAreaElement;
    // Set overflowY to hidden and height to fit-content
    // so we can get a correct scrollHeight
    ta.style.overflowY = 'hidden';
    ta.style.height = 'fit-content';
    const { scrollHeight } = ta;
    if (scrollHeight > HEIGHT_LIMIT) {
      // Enable scroll when height limitation is reached
      ta.style.overflowY = 'scroll';
      ta.style.height = `${HEIGHT_LIMIT}px`;
    } else {
      // set overflowY back to hidden when height limitation is not reached
      ta.style.overflowY = 'hidden';
      // Set height to scrollHeight
      ta.style.height = `${scrollHeight}px`;
    }
  };

  const onClick = () => {
    const promptStr = promptRef.current?.value ?? '';
    if (promptStr.trim().length === 0) {
      toast.error(
        'You prompt is blank. Blank prompt is a waste of your tokens quota.'
      );
    } else {
      createMsgMutation.mutate(
        {
          conversationId,
          role: MESSAGE_USER,
          content: promptStr,
        },
        {
          onSuccess(message) {
            // Update cache
            queryClient.setQueryData<Message[]>(
              [...LIST_MESSAGES_KEY, { conversationId }],
              (messages) => (messages ? [...messages, message] : [message])
            );
          },
        }
      );
      if (promptRef.current) {
        promptRef.current.value = '';
      }
    }
  };

  return (
    <div className="mb-4 flex min-h-16 w-auto items-end border-b-2 border-slate-500 text-sm">
      <div className="mb-5 grow">
        <Textarea
          placeholder="How can I help?"
          className="resize-none overflow-y-hidden border-0 px-2"
          rows={1}
          onChange={onChange}
          ref={promptRef}
        />
      </div>
      <Button className="mb-5" onClick={onClick}>
        <PaperPlaneIcon />
      </Button>
    </div>
  );
}
