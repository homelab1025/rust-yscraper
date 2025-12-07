import React, { useEffect, useMemo, useState } from 'react';

export type PaginationatProps = {
  total: number;
  pageSize?: number;
  onPageChange?: (page: number) => void;
};

export default function Paginationat({ total, pageSize = 10, onPageChange }: PaginationatProps) {
  const [page, setPage] = useState<number>(1);

  const totalPages = useMemo(() => Math.ceil(total / pageSize), [total, pageSize]);
  const pagesToShow = useMemo(() => {
    const tp = totalPages;
    if (tp <= 7) return Array.from({ length: tp }, (_, i) => i + 1);
    const start = Math.max(1, page - 2);
    const end = Math.min(tp, start + 4);
    return Array.from({ length: end - start + 1 }, (_, i) => start + i);
  }, [page, totalPages]);

  // Clamp current page when total or pageSize changes
  useEffect(() => {
    if (totalPages === 0 && page !== 1) {
      setPage(1);
    } else if (page > totalPages && totalPages > 0) {
      setPage(totalPages);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [totalPages]);

  // Notify parent when page changes
  useEffect(() => {
    onPageChange?.(page);
  }, [onPageChange, page]);

  return (
    <div className="pagination" role="navigation" aria-label="Pagination">
      <button disabled={page === 1} onClick={() => setPage((p) => Math.max(1, p - 1))}>
        Prev
      </button>
      {pagesToShow.map((p) => (
        <button key={p} className={["page-btn", p === page ? 'active' : ''].join(' ')} onClick={() => setPage(p)}>
          {p}
        </button>
      ))}
      <button disabled={page === totalPages || totalPages === 0} onClick={() => setPage((p) => Math.min(totalPages || 1, p + 1))}>
        Next
      </button>
      <span className="total">Total: {total}</span>
    </div>
  );
}
