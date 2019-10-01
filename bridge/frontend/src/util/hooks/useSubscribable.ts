import { Subscribable } from 'rxjs';
import { useState, useEffect, useMemo } from 'react';
import getErrorMsg from '~util/getErrorMsg';
import React from 'react';

type Meta = {
  loaded: boolean;
  error: string | null;
  updatedAt: number;
}

type Result<T> = [T, Meta];

function useSubscribable<T>(getTarget: () => Subscribable<T>, deps: any[]): Result<T | undefined>;
function useSubscribable<T>(getTarget: () => Subscribable<T>, deps: any[], fallback: T): Result<T>;
function useSubscribable<T>(getTarget: () => Subscribable<T>, deps: any[], fallback?: T): Result<T | undefined> {
  const [loaded, setLoaded] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [updatedAt, setUpdatedAt] = useState(() => Date.now());
  const [value, setValue] = useState<T | undefined>(fallback);

  const resetState = React.useCallback(() => {
    setLoaded(false);
    setError(null);
    setUpdatedAt(Date.now());
    setValue(fallback);
  }, [fallback]);

  const target = useMemo(getTarget, deps);

  useEffect(() => {
    resetState();

    const subscribtion = target.subscribe({
      next: value => {
        setLoaded(true);
        setError(null);
        setUpdatedAt(Date.now());
        setValue(value);
      },
      error: err => {
        setLoaded(true);
        setError(getErrorMsg(err))
      },
    });

    return () => subscribtion.unsubscribe();
  }, [target]);

  const meta: Meta = useMemo(() => ({
    loaded, updatedAt, error,
  }), [loaded, updatedAt, error]);

  return [value, meta];
}

export { useSubscribable };
