import type { UnlistenFn } from '@tauri-apps/api/event';
import { emit, listen } from '@tauri-apps/api/event';
import { useEffect, useRef, useState } from 'react';

import {
  STREAM_DONE,
  STREAM_ERROR,
  STREAM_START,
  STREAM_STOPPED,
} from '@/lib/constants';
import log from '@/lib/log';

import ChatMessage from './ChatMessage';
import { useToast } from './ui/use-toast';

type Props = {
  onMessageReceived: (message: string) => void;
};

export function BotMessageReceiver({ onMessageReceived }: Props) {
  const [receiving, setReceiving] = useState(false);
  const acceptingRef = useRef<boolean>(false);
  const [activeBotMessage, setActiveBotMessage] = useState('');
  const listenerRef = useRef<UnlistenFn>();
  const { toast } = useToast();

  const startStreaming = () => {
    setReceiving(true);
    acceptingRef.current = true;
  };

  const endStreaming = () => {
    setReceiving(false);
    acceptingRef.current = false;
  };

  const bindListener = async () => {
    listenerRef.current = await listen<string>('bot-reply', (event) => {
      const nextMsg = event.payload;
      switch (true) {
        case nextMsg === STREAM_START:
          startStreaming();
          break;
        case nextMsg === STREAM_DONE:
          endStreaming();
          break;
        case nextMsg === STREAM_STOPPED:
          endStreaming();
          break;
        case nextMsg.startsWith(STREAM_ERROR):
          toast({
            variant: 'destructive',
            title: 'Bot Error',
            description: nextMsg.split(STREAM_ERROR).at(-1),
          });
          endStreaming();
          break;
        default:
          if (acceptingRef.current) {
            setActiveBotMessage((state) => {
              return `${state}${nextMsg}`;
            });
          }
          break;
      }
    });
    await log.info('Listener is bound');
  };

  const unbindListener = async () => {
    if (listenerRef.current) {
      listenerRef.current();
      listenerRef.current = undefined;
      await log.info('Listener is unbound');
    }
  };

  useEffect(() => {
    // stop bot when entering the page
    // in case it was left running before
    emit('stop-bot');
    bindListener();
    return () => {
      unbindListener();
      // stop bot when leaving the page
      emit('stop-bot');
    };
  }, []);

  useEffect(() => {
    if (!receiving && activeBotMessage.length > 0) {
      onMessageReceived(activeBotMessage);
      setActiveBotMessage('');
    }
  }, [activeBotMessage, receiving]);

  const render = () => {
    return activeBotMessage.length > 0 ? (
      <ChatMessage.BotReceiving message={activeBotMessage} />
    ) : (
      <ChatMessage.BotLoading />
    );
  };

  return receiving ? render() : null;
}
