// Reuses the pre-React loading styles from index.html,
// so the visual is seamless: HTML spinner → React spinner → actual content.
export const PopoverPendingLayout = () => (
  <div className='pre-react-loading'>
    <div className='pre-react-spinner' />
  </div>
);
