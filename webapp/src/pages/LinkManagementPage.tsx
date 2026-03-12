import * as React from 'react';
import { useEffect, useState } from 'react';
import { Link } from 'react-router-dom';
import SummaryStats from '../components/SummaryStats';
import StatusBadge from '../components/StatusBadge';
import { type LinkDto, CommentState } from '../api-client';
import { useServices } from '../contexts/ServicesContext';

const monthNames = [
  "January", "February", "March", "April", "May", "June",
  "July", "August", "September", "October", "November", "December"
];

const formatThreadMetadata = (month?: number | null, year?: number | null, fallback?: string) => {
  if (month && year) {
    return `${monthNames[month - 1]} ${year}`;
  }
  return fallback;
};

const isCompleted = (link: LinkDto) =>
  link.total_comment_count > 0 && link.picked_comment_count >= link.total_comment_count;

export default function LinkManagementPage(): React.JSX.Element {
  const { linksApi } = useServices();
  const [links, setLinks] = useState<LinkDto[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [deleteConfirm, setDeleteConfirm] = useState<number | null>(null);

  const fetchLinks = async () => {
    try {
      setLoading(true);
      const response = await linksApi.listLinks();
      setLinks(response.data);
      setError(null);
    } catch (err) {
      setError('Failed to fetch links');
      console.error(err);
    } finally {
      setLoading(false);
    }
  };

  const handleDelete = async (id: number) => {
    try {
      await linksApi.deleteLink(id);
      await fetchLinks();
    } catch (err) {
      setError('Failed to delete link');
      console.error(err);
    } finally {
      setDeleteConfirm(null);
    }
  };

  useEffect(() => {
    fetchLinks();
  }, []);

  // TODO: when server will return the discarded comments per link use that to calculate the totalreviewed = total count - picked count - discarded count
  const totalPicked = links.reduce((sum, l) => sum + (l.picked_comment_count ?? 0), 0);
  const totalReviewed = totalPicked;

  return (
    <div className="max-w-6xl mx-auto flex flex-col gap-8">
      <SummaryStats reviewed={totalReviewed} picked={totalPicked} discarded={0} />

      {error && (
        <div className="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg text-sm">
          {error}
        </div>
      )}

      <div className="bg-white rounded-xl shadow-sm border border-slate-200 overflow-hidden">
        <div className="px-6 py-5 border-b border-slate-200 flex justify-between items-center">
          <h2 className="text-slate-900 text-xl font-bold tracking-tight">Task Control Center</h2>
          <span className="text-xs text-slate-500 bg-slate-100 px-2.5 py-1 rounded-full flex items-center gap-1">
            <span className="w-1.5 h-1.5 rounded-full bg-primary animate-pulse"></span>
            Live Update
          </span>
        </div>

        <div className="overflow-x-auto">
          <table className="w-full text-left border-collapse">
            <thead>
              <tr className="bg-slate-50">
                <th className="px-6 py-4 text-xs font-bold uppercase tracking-wider text-slate-500">ID</th>
                <th className="px-6 py-4 text-xs font-bold uppercase tracking-wider text-slate-500">Month</th>
                <th className="px-6 py-4 text-xs font-bold uppercase tracking-wider text-slate-500">Date Added</th>
                <th className="px-6 py-4 text-xs font-bold uppercase tracking-wider text-slate-500">Comments (Picked / Total)</th>
                <th className="px-6 py-4 text-xs font-bold uppercase tracking-wider text-slate-500">Status</th>
                <th className="px-6 py-4 text-xs font-bold uppercase tracking-wider text-slate-500 text-center">Actions</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-100">
              {loading ? (
                <tr>
                  <td colSpan={6} className="px-6 py-12 text-center text-slate-500">
                    <span className="material-symbols-outlined animate-spin !text-3xl">refresh</span>
                  </td>
                </tr>
              ) : links.length === 0 ? (
                <tr>
                  <td colSpan={6} className="px-6 py-12 text-center text-slate-500 text-sm">
                    No links found. Add a Hacker News item ID in the sidebar.
                  </td>
                </tr>
              ) : (
                links.map((link) => (
                  <tr key={link.id} className="hover:bg-slate-50 transition-colors">
                    <td className="px-6 py-4 text-sm font-medium text-slate-900">#{link.id}</td>
                    <td className="px-6 py-4 text-sm">
                      <a
                        href={link.url}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="text-primary hover:underline font-semibold"
                      >
                        {formatThreadMetadata(link.thread_month, link.thread_year, link.url)}
                      </a>
                    </td>
                    <td className="px-6 py-4 text-sm text-slate-600">
                      {new Date(link.date_added).toLocaleDateString()}
                    </td>
                    <td className="px-6 py-4 text-sm text-slate-600">
                      <span className="font-bold text-slate-900">{link.picked_comment_count}</span>
                      {' '}/ {link.total_comment_count}
                    </td>
                    <td className="px-6 py-4">
                      <StatusBadge completed={isCompleted(link)} />
                    </td>
                    <td className="px-6 py-4">
                      <div className="flex items-center justify-center gap-3">
                        <Link
                          to={`/comments?url_id=${link.id}&state=${CommentState.New}`}
                          className="text-slate-400 hover:text-slate-900 transition-colors"
                          title="View Remaining"
                        >
                          <span className="material-symbols-outlined !text-xl">format_list_bulleted</span>
                        </Link>
                        <Link
                          to={`/comments?url_id=${link.id}&state=${CommentState.Picked}`}
                          className="text-slate-400 hover:text-emerald-500 transition-colors"
                          title="View Picked"
                        >
                          <span className="material-symbols-outlined !text-xl">check_circle</span>
                        </Link>
                        <Link
                          to={`/comments?url_id=${link.id}&state=${CommentState.Discarded}`}
                          className="text-slate-400 hover:text-rose-500 transition-colors"
                          title="View Discarded"
                        >
                          <span className="material-symbols-outlined !text-xl">cancel</span>
                        </Link>
                        {deleteConfirm === link.id ? (
                          <div className="flex items-center gap-1">
                            <button
                              onClick={() => handleDelete(link.id)}
                              className="text-xs text-red-600 font-semibold hover:text-red-700"
                            >
                              Confirm
                            </button>
                            <button
                              onClick={() => setDeleteConfirm(null)}
                              className="text-xs text-slate-500 hover:text-slate-700"
                            >
                              Cancel
                            </button>
                          </div>
                        ) : (
                          <button
                            onClick={() => setDeleteConfirm(link.id)}
                            className="text-slate-400 hover:text-red-600 transition-colors"
                            title="Delete"
                          >
                            <span className="material-symbols-outlined !text-xl">delete</span>
                          </button>
                        )}
                      </div>
                    </td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>

        <div className="px-6 py-4 bg-slate-50 border-t border-slate-200 flex justify-between items-center text-xs text-slate-500">
          <span>Showing {links.length} active item tracking node{links.length !== 1 ? 's' : ''}</span>
        </div>
      </div>
    </div>
  );
}
