import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import type { CommentDto, CommentsPage } from '../types';

function useApiBase() {
  const base = import.meta.env.VITE_API as string | undefined;
  return base ?? '';
}

export type CommentsProps = {
  page: number;
  pageSize?: number;
  onTotalChange?: (total: number) => void;
};

export default function Comments({ page, pageSize = 10, onTotalChange }: CommentsProps) {
  const apiBase = useApiBase();

  const [items, setItems] = useState<CommentDto[]>([]);
  const [total, setTotal] = useState<number>(0);
  const [loading, setLoading] = useState<boolean>(false);

  const [selectedIndex, setSelectedIndex] = useState<number | null>(null);
  const [marks, setMarks] = useState<Record<number, 'blue' | 'red'>>({});

  const scrollerRef = useRef<HTMLDivElement | null>(null);
  const rowRefs = useRef<Array<HTMLTableRowElement | null>>([]);

  const setRowRef = useCallback((idx: number) => (el: HTMLTableRowElement | null) => {
    rowRefs.current[idx] = el;
  }, []);

  const totalPages = useMemo(() => Math.ceil(total / pageSize), [total, pageSize]);

  const ensureRowInView = useCallback((idx: number | null, buffer = 2) => {
    if (idx == null) return;
    const container = scrollerRef.current;
    if (!container) return;

    const topIdx = Math.max(0, idx - buffer);
    const botIdx = Math.min(items.length - 1, idx + buffer);
    const topEl = rowRefs.current[topIdx];
    const botEl = rowRefs.current[botIdx] || rowRefs.current[idx];
    if (!topEl || !botEl) return;

    const desiredTop = topEl.offsetTop;
    const desiredBottom = botEl.offsetTop + botEl.offsetHeight;

    const viewTop = container.scrollTop;
    const viewBottom = viewTop + container.clientHeight;

    let newTop: number | null = null;
    if (desiredTop < viewTop) newTop = desiredTop;
    else if (desiredBottom > viewBottom) newTop = desiredBottom - container.clientHeight;
    if (newTop != null) container.scrollTo({ top: Math.max(0, newTop), behavior: 'smooth' });
  }, [items.length]);

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const offset = (page - 1) * pageSize;
      const resp = await fetch(`${apiBase}/comments?offset=${offset}&count=${pageSize}`);
      if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
      const data: CommentsPage = await resp.json();
      rowRefs.current = [];
      setItems(data.items ?? []);
      setTotal(data.total ?? 0);
      onTotalChange?.(data.total ?? 0);

      if (scrollerRef.current) scrollerRef.current.scrollTop = 0;
    } catch (e) {
      // eslint-disable-next-line no-console
      console.error('Failed to load comments', e);
      setItems([]);
      setTotal(0);
      onTotalChange?.(0);
      setSelectedIndex(null);
    } finally {
      setLoading(false);
    }
  }, [apiBase, onTotalChange, page, pageSize]);

  useEffect(() => {
    load();
  }, [load]);

  useEffect(() => {
    if (items.length > 0) {
      setSelectedIndex((prev) => {
        if (prev == null) return 0;
        if (prev >= items.length) return items.length - 1;
        return prev;
      });
    } else {
      setSelectedIndex(null);
    }
    const id = requestAnimationFrame(() => ensureRowInView(selectedIndex));
    return () => cancelAnimationFrame(id);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [items]);

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      const activeTag = (document.activeElement as HTMLElement | null)?.tagName;
      if (activeTag && ['INPUT', 'TEXTAREA', 'SELECT'].includes(activeTag)) return;
      if (loading) return;

      if (e.key === 'j') {
        if (!items.length) return;
        setSelectedIndex((prev) => {
          const next = prev == null ? 0 : Math.min(items.length - 1, prev + 1);
          return next;
        });
        e.preventDefault();
        setTimeout(() => ensureRowInView(selectedIndex), 0);
      } else if (e.key === 'k') {
        if (!items.length) return;
        setSelectedIndex((prev) => {
          const next = prev == null ? 0 : Math.max(0, prev - 1);
          return next;
        });
        e.preventDefault();
        setTimeout(() => ensureRowInView(selectedIndex), 0);
      } else if (e.key === 'a') {
        if (selectedIndex != null && items[selectedIndex]) {
          const id = items[selectedIndex].id;
          setMarks((m) => ({ ...m, [id]: 'blue' }));
          e.preventDefault();
        }
      } else if (e.key === 'd') {
        if (selectedIndex != null && items[selectedIndex]) {
          const id = items[selectedIndex].id;
          setMarks((m) => ({ ...m, [id]: 'red' }));
          e.preventDefault();
        }
      }
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, [ensureRowInView, items, loading, selectedIndex]);

  const rowClass = (c: CommentDto, idx: number) => {
    const cls: string[] = [];
    if (selectedIndex === idx) cls.push('selected');
    const m = marks[c.id];
    if (m === 'blue') cls.push('mark-blue');
    if (m === 'red') cls.push('mark-red');
    return cls.join(' ');
  };

  return (
    <>
      <h2>Not Filtered Comments</h2>
      {loading ? (
        <div className="loading">Loading...</div>
      ) : (
        <div ref={scrollerRef} className="table-scroller">
          <table className="comments-table" aria-label="Comments table (use j/k to move, a to mark blue, d to mark red)">
            <thead>
              <tr>
                <th>Text</th>
                <th>User</th>
                <th>URL ID</th>
                <th>Date</th>
              </tr>
            </thead>
            <tbody>
              {items.map((c, idx) => (
                <tr
                  key={c.id}
                  className={rowClass(c, idx)}
                  aria-selected={selectedIndex === idx}
                  onClick={() => {
                    setSelectedIndex(idx);
                    ensureRowInView(idx);
                  }}
                  ref={setRowRef(idx)}
                >
                  <td className="text">{c.text}</td>
                  <td>{c.user}</td>
                  <td>{c.url_id}</td>
                  <td>{c.date}</td>
                </tr>
              ))}
              {items.length === 0 && (
                <tr>
                  <td colSpan={4} className="empty">
                    No comments
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      )}

      <div className="legend" aria-hidden="true">
        Keys: j = down, k = up, a = mark blue, d = mark red
      </div>
    </>
  );
}
