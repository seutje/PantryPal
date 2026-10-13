const paths = {
  Week: 'M5 3v4m14-4v4M3 10h18M5 5h14a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V7a2 2 0 0 1 2-2Z',
  Meals: 'M4 3v6a3 3 0 0 0 6 0V3M7 3v18M19 21V3c-4 3-4 9 0 9',
  Shopping: 'M3 3h2l3 13h11l2-9H6M9 21h.01M18 21h.01',
  Ingredients:
    'M8 3h8v4H8zM8 7c-3 1-3 3-3 5v7a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2v-7c0-2 0-4-3-5M5 12h14',
  Settings:
    'M12 8a4 4 0 1 0 0 8 4 4 0 0 0 0-8ZM9 3h6l1 3 3 1 2 5-2 5-3 1-1 3H9l-1-3-3-1-2-5 2-5 3-1 1-3Z',
};
export function NavIcon({ name }: { name: keyof typeof paths }) {
  return (
    <svg
      width="22"
      height="22"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.7"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      <path d={paths[name]} />
    </svg>
  );
}
