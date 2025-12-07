import React, { useState } from 'react';
import Comments from './Comments';
import Paginationat from './Paginationat';

export default function ListComments() {
  // Mirror values to connect child components together.
  const [pageFromPager, setPageFromPager] = useState<number>(1);
  const [totalFromComments, setTotalFromComments] = useState<number>(0);

  return (
    <section className="table-wrap">
      <Comments page={pageFromPager} onTotalChange={setTotalFromComments} />
      <Paginationat total={totalFromComments} onPageChange={setPageFromPager} />
    </section>
  );
}
